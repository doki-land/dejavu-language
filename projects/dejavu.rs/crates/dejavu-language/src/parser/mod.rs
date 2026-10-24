//! Hand-written recursive-descent template parser.

mod expr;
mod stmt;
mod stmt_head;

use dejavu_diagnostic::codes;
use dejavu_ir::types::Span;
use dejavu_ir::{IrDocument, Language, NativeKind, NativeNode, NativeTemplate, TemplateConfig, default_language, encode_native};
use expr::parse_expr;
use stmt_head::{CodeHead, classify_code, head_stop_kind};

use crate::error::{ParseError, ParseResult};
use crate::lexer::TrimMode;

pub struct Parser<'a> {
    pub(crate) source: &'a str,
    pub(crate) file: &'a str,
    pub(crate) delimiters: Delimiters,
}

/// Delimiter configuration extracted from `Language.template`.
#[derive(Debug, Clone)]
pub(crate) struct Delimiters {
    pub(crate) code_start: String,
    pub(crate) code_end: String,
    pub(crate) comment_start: String,
    pub(crate) comment_end: String,
}

impl From<&TemplateConfig> for Delimiters {
    fn from(t: &TemplateConfig) -> Self {
        Self {
            code_start: t.code_start.clone(),
            code_end: t.code_end.clone(),
            comment_start: t.comment_start.clone(),
            comment_end: t.comment_end.clone(),
        }
    }
}

impl Delimiters {
    /// Byte length of the code start delimiter (for fast `.starts_with` checks).
    pub(crate) fn code_start_len(&self) -> usize { self.code_start.len() }
    pub(crate) fn code_end_len(&self) -> usize { self.code_end.len() }
    pub(crate) fn comment_start_len(&self) -> usize { self.comment_start.len() }
    pub(crate) fn comment_end_len(&self) -> usize { self.comment_end.len() }
}

/// One scanned code tag: interior text plus the trim markers on both sides.
pub(crate) struct TagScan {
    pub(crate) inner: String,
    pub(crate) inner_range: std::ops::Range<usize>,
    pub(crate) next: usize,
    pub(crate) trim_left: TrimMode,
    pub(crate) trim_right: TrimMode,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str, file: &'a str) -> Self {
        Self::with_language(source, file, &default_language())
    }

    pub fn with_language(source: &'a str, file: &'a str, language: &Language) -> Self {
        Self { source, file, delimiters: (&language.template).into() }
    }

    pub fn parse_template(&self) -> Result<NativeTemplate, ParseError> {
        let (children, _) = self.parse_body(0, &[])?;
        self.validate_partial_layout(&children)?;
        self.validate_macro_layout(&children)?;
        Ok(NativeTemplate { children })
    }

    /// A template declaring `partial` must consist of exactly one partial:
    /// no other statements, no surrounding content except whitespace/comments,
    /// and no partial nested inside another control-flow body.
    fn validate_partial_layout(&self, children: &[NativeNode]) -> Result<(), ParseError> {
        let mut declared: Option<&NativeNode> = None;
        for child in children {
            if let NativeKind::Partial { .. } = &child.kind {
                if declared.is_some() {
                    return Err(self.partial_layout_error(
                        child,
                        "only one `partial` declaration is allowed per template",
                    ));
                }
                declared = Some(child);
            }
        }

        // No partial nested inside another control-flow body or another partial.
        for child in children {
            match &child.kind {
                NativeKind::Partial { body, .. } => {
                    if let Some(node) = Self::find_partial(body) {
                        return Err(self.partial_layout_error(node, "`partial` cannot be nested"));
                    }
                }
                _ => {
                    if let Some(node) = Self::find_partial(std::slice::from_ref(child)) {
                        return Err(self.partial_layout_error(node, "`partial` cannot be nested"));
                    }
                }
            }
        }

        // Layout rules only constrain templates that actually declare a partial.
        if declared.is_none() {
            return Ok(());
        }
        for child in children {
            match &child.kind {
                NativeKind::Partial { .. } | NativeKind::Comment(_) => {}
                NativeKind::Text(value) if value.trim().is_empty() => {}
                _ => {
                    return Err(self.partial_layout_error(
                        child,
                        "`partial` must wrap the whole template; other top-level content is not allowed",
                    ));
                }
            }
        }
        Ok(())
    }

    /// `macro` declarations are template-level only: no nesting inside control
    /// flow or another macro, no duplicate names, no control keywords as names.
    fn validate_macro_layout(&self, children: &[NativeNode]) -> Result<(), ParseError> {
        let mut seen: Vec<&str> = Vec::new();
        for child in children {
            match &child.kind {
                NativeKind::Macro { name, .. } => {
                    if matches!(
                        name.as_str(),
                        "if" | "else" | "for" | "loop" | "block" | "partial" | "macro" | "end" | "include"
                            | "extends" | "super" | "raw"
                    ) {
                        return Err(self.partial_layout_error(child, "macro name cannot be a control keyword"));
                    }
                    if seen.contains(&name.as_str()) {
                        return Err(self.partial_layout_error(child, "duplicate `macro` name in one template"));
                    }
                    seen.push(name.as_str());
                }
                NativeKind::Import { names, .. } => {
                    for imported in names {
                        let local = imported.alias.as_deref().unwrap_or(&imported.name);
                        if seen.contains(&local) {
                            return Err(self.partial_layout_error(
                                child,
                                "duplicate macro/import name in one template",
                            ));
                        }
                        seen.push(local);
                    }
                }
                _ => {
                    if let Some(node) = Self::find_macro(std::slice::from_ref(child)) {
                        return Err(self.partial_layout_error(node, "`macro` must be a top-level declaration"));
                    }
                }
            }
        }
        Ok(())
    }

    fn partial_layout_error(&self, node: &NativeNode, message: &str) -> ParseError {
        let (start, end) = node.span.as_ref().map(|s| (s.start, s.end)).unwrap_or((0, 1));
        ParseError::at(self.source, self.file, start, end.saturating_sub(start).max(1), message, "unexpected")
            .with_code(codes::PARSE_UNEXPECTED_TOKEN)
    }

/// Find the first `partial` declaration anywhere inside a sibling list.
///
/// Used by layout validation: a `partial` may only be a top-level statement of
/// its own template, never nested inside control flow.
fn find_partial(nodes: &[NativeNode]) -> Option<&NativeNode> {
    for node in nodes {
        match &node.kind {
            NativeKind::Partial { .. } => return Some(node),
            NativeKind::If { consequent, else_ifs, alternate, .. } => {
                if let Some(hit) = Self::find_partial(consequent) {
                    return Some(hit);
                }
                for (_, body) in else_ifs {
                    if let Some(hit) = Self::find_partial(body) {
                        return Some(hit);
                    }
                }
                if let Some(body) = alternate {
                    if let Some(hit) = Self::find_partial(body) {
                        return Some(hit);
                    }
                }
            }
            NativeKind::For { body, .. } | NativeKind::Block { body, .. } | NativeKind::Macro { body, .. } => {
                if let Some(hit) = Self::find_partial(body) {
                    return Some(hit);
                }
            }
            _ => {}
        }
    }
    None
}

/// Find the first `macro` declaration or `from ... import` inside a control-flow body.
fn find_macro(nodes: &[NativeNode]) -> Option<&NativeNode> {
    for node in nodes {
        match &node.kind {
            NativeKind::Macro { .. } | NativeKind::Import { .. } => return Some(node),
            NativeKind::If { consequent, else_ifs, alternate, .. } => {
                if let Some(hit) = Self::find_macro(consequent) {
                    return Some(hit);
                }
                for (_, body) in else_ifs {
                    if let Some(hit) = Self::find_macro(body) {
                        return Some(hit);
                    }
                }
                if let Some(body) = alternate {
                    if let Some(hit) = Self::find_macro(body) {
                        return Some(hit);
                    }
                }
            }
            NativeKind::For { body, .. } | NativeKind::Block { body, .. } | NativeKind::Partial { body, .. } => {
                if let Some(hit) = Self::find_macro(body) {
                    return Some(hit);
                }
            }
            _ => {}
        }
    }
    None
}

    /// Peek at a code block starting at index `i`.
    /// Returns `(code_inner, content_span_in_source, index_after_close)`.
    /// Uses configured delimiters.
    pub(crate) fn peek_code(&self, i: usize) -> Result<(String, std::ops::Range<usize>, usize), ParseError> {
        let tag = self.scan_tag(i)?;
        Ok((tag.inner, tag.inner_range, tag.next))
    }

    /// Scan one code tag, including the whitespace-control markers on both ends.
    pub(crate) fn scan_tag(&self, i: usize) -> Result<TagScan, ParseError> {
        let ds = &self.delimiters;
        if !self.source[i..].starts_with(&ds.code_start) {
            return Err(ParseError::at(
                self.source,
                self.file,
                i,
                1,
                format!("expected code open `{}`", ds.code_start),
                "here",
            )
            .with_code(codes::PARSE_UNEXPECTED_TOKEN));
        }
        let start_len = ds.code_start_len();
        let end_len = ds.code_end_len();
        let mut j = i + start_len;
        let b = self.source.as_bytes();
        // Trim markers: check the first byte after code start for `.-_~=`
        let mut trim_left = TrimMode::None;
        if j < self.source.len() {
            if let Some(t) = TrimMode::from_byte(b[j]) {
                trim_left = t;
                j += 1;
            }
        }
        let rest = &self.source[j..];
        let end = rest.find(&ds.code_end).ok_or_else(|| {
            ParseError::at(
                self.source,
                self.file,
                i,
                start_len,
                "unclosed code block",
                "opens here",
            )
            .with_code(codes::PARSE_UNCLOSED_BLOCK)
        })?;
        // Closing marker: the byte right before the close delimiter.
        let mut inner_end = j + end;
        let mut trim_right = TrimMode::None;
        if inner_end > j {
            if let Some(t) = TrimMode::from_byte(b[inner_end - 1]) {
                trim_right = t;
                inner_end -= 1;
            }
        }
        let inner = self.source[j..inner_end].to_string();
        Ok(TagScan { inner, inner_range: j..inner_end, next: j + end + end_len, trim_left, trim_right })
    }

    /// Parse until a stop head is encountered.
    pub(crate) fn parse_until(&self, i: usize, stop: &[&str]) -> Result<(Vec<NativeNode>, usize), ParseError> {
        self.parse_body(i, stop)
    }

    fn parse_body(&self, mut i: usize, stop: &[&str]) -> Result<(Vec<NativeNode>, usize), ParseError> {
        let mut children = Vec::new();
        let bytes = self.source.as_bytes();
        let ds = &self.delimiters;
        let cs_len = ds.code_start_len();
        let cms_len = ds.comment_start_len();
        let cme_len = ds.comment_end_len();
        let cs = &ds.code_start;
        let cms = &ds.comment_start;
        let cme = &ds.comment_end;
        // Right-side trim of the previous tag, waiting for the next text node.
        let mut pending: Option<&'static str> = None;

        while i < self.source.len() {
            if self.source[i..].starts_with(cs) && !stop.is_empty() {
                let tag = self.scan_tag(i)?;
                let code = tag.inner.trim();
                let content_base = tag.inner_range.start + (tag.inner.len() - tag.inner.trim_start().len());
                let head = classify_code(self.source, self.file, code, content_base)?;
                if should_stop_head(&head, stop) {
                    reject_closer_trim(self.source, self.file, &tag, &head)?;
                    return Ok((children, i));
                }
            }

            if self.source[i..].starts_with(cms) {
                if let Some(end) = self.source[i + cms_len..].find(cme) {
                    let inner = &self.source[i + cms_len..i + cms_len + end];
                    let next = i + cms_len + end + cme_len;
                    children.push(NativeNode::new(
                        NativeKind::Comment(inner.to_string()),
                        Some(Span { start: i, end: next }),
                    ));
                    i = next;
                    continue;
                }
                return Err(ParseError::at(self.source, self.file, i, cms_len, "unclosed comment", "comment starts here")
                    .with_code(codes::PARSE_UNCLOSED_BLOCK));
            }

            if self.source[i..].starts_with(cs) {
                let open = i;
                let after_open = i + cs_len;
                // `<%!` escapes the code open delimiter as literal text.
                if after_open < self.source.len()
                    && TrimMode::from_byte(bytes[after_open]).is_none()
                    && bytes[after_open] == b'!'
                {
                    push_text(&mut children, cs.clone(), i, after_open + 1, pending.take());
                    i = after_open + 1;
                    continue;
                }

                let tag = self.scan_tag(open)?;
                let code = tag.inner.trim();
                let code_content_start = tag.inner_range.start + (tag.inner.len() - tag.inner.trim_start().len());
                let next_i = tag.next;
                let head = classify_code(self.source, self.file, code, code_content_start)?;

                if !stop.is_empty() && should_stop_head(&head, stop) {
                    reject_closer_trim(self.source, self.file, &tag, &head)?;
                    return Ok((children, open));
                }

                match head {
                    CodeHead::If { test_src, test_base } => {
                        let (node, new_i) =
                            self.parse_if(open, next_i, test_src, test_base, tag.trim_left, tag.trim_right, pending.take())?;
                        children.push(node);
                        i = new_i;
                    }
                    CodeHead::Loop { item, iter_src, iter_base } => {
                        let (node, new_i) = self.parse_loop(
                            open,
                            next_i,
                            item,
                            iter_src,
                            iter_base,
                            tag.trim_left,
                            tag.trim_right,
                            pending.take(),
                        )?;
                        children.push(node);
                        i = new_i;
                    }
                    CodeHead::Extends { parent_src, parent_base } => {
                        let parent = parse_expr(self.source, self.file, parent_base, parent_src)?;
                        children.push(NativeNode::new(
                            NativeKind::Extends { parent, trim: tag.trim_left.as_str().into() },
                            Some(Span { start: open, end: next_i }),
                        ));
                        pending = right_trim(tag.trim_right);
                        i = next_i;
                    }
                    CodeHead::Include { path_src, path_base, with_src, with_base, only } => {
                        let path = parse_expr(self.source, self.file, path_base, path_src)?;
                        let with = match with_src {
                            Some(src) => Some(parse_expr(self.source, self.file, with_base, src)?),
                            None => None,
                        };
                        children.push(NativeNode::new(
                            NativeKind::Include { path, with, only, trim: tag.trim_left.as_str().into() },
                            Some(Span { start: open, end: next_i }),
                        ));
                        pending = right_trim(tag.trim_right);
                        i = next_i;
                    }
                    CodeHead::Block { name } => {
                        let (node, new_i) =
                            self.parse_block(open, next_i, name, tag.trim_left, tag.trim_right, pending.take())?;
                        children.push(node);
                        i = new_i;
                    }
                    CodeHead::Partial { params_src, params_base } => {
                        let (node, new_i) = self.parse_partial(
                            open,
                            next_i,
                            params_src,
                            params_base,
                            tag.trim_left,
                            tag.trim_right,
                            pending.take(),
                        )?;
                        children.push(node);
                        i = new_i;
                    }
                    CodeHead::From { path, names } => {
                        let (node, new_i) = self.parse_import(
                            open,
                            next_i,
                            path,
                            names,
                            tag.trim_left,
                            tag.trim_right,
                            pending.take(),
                        );
                        children.push(node);
                        i = new_i;
                    }
                    CodeHead::Macro { name, params_src, params_base } => {
                        let (node, new_i) = self.parse_macro(
                            open,
                            next_i,
                            name,
                            params_src,
                            params_base,
                            tag.trim_left,
                            tag.trim_right,
                            pending.take(),
                        )?;
                        children.push(node);
                        i = new_i;
                    }
                    CodeHead::Super => {
                        children.push(NativeNode::new(
                            NativeKind::Super { trim: tag.trim_left.as_str().into() },
                            Some(Span { start: open, end: next_i }),
                        ));
                        pending = right_trim(tag.trim_right);
                        i = next_i;
                    }
                    CodeHead::EndIf
                    | CodeHead::EndLoop
                    | CodeHead::EndBlock
                    | CodeHead::EndPartial
                    | CodeHead::EndMacro
                    | CodeHead::Else
                    | CodeHead::ElseIf { .. } => {
                        reject_closer_trim(self.source, self.file, &tag, &head)?;
                        if stop.is_empty() {
                            let label = match &head {
                                CodeHead::EndIf => "end if",
                                CodeHead::EndLoop => "end loop",
                                CodeHead::EndBlock => "end block",
                                CodeHead::EndPartial => "end partial",
                                CodeHead::EndMacro => "end macro",
                                CodeHead::Else => "else",
                                CodeHead::ElseIf { .. } => "else if",
                                _ => "control",
                            };
                            return Err(ParseError::at(
                                self.source,
                                self.file,
                                code_content_start,
                                code.len().max(1),
                                format!("unexpected `{label}`"),
                                "unexpected",
                            )
                            .with_code(codes::PARSE_UNEXPECTED_TOKEN));
                        }
                        return Ok((children, open));
                    }
                    CodeHead::Expr { src, base } => {
                        let expr = parse_expr(self.source, self.file, base, src)?;
                        children.push(NativeNode::new(
                            NativeKind::Interpolation {
                                expression: expr,
                                trim: tag.trim_left.as_str().into(),
                                raw: false,
                            },
                            Some(Span { start: open, end: next_i }),
                        ));
                        pending = right_trim(tag.trim_right);
                        i = next_i;
                    }
                }
                continue;
            }

            let next_code = self.source[i..].find(cs).map(|p| i + p);
            let next_comment = self.source[i..].find(cms).map(|p| i + p);
            let next = match (next_code, next_comment) {
                (Some(a), Some(b)) => Some(a.min(b)),
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (None, None) => None,
            };
            match next {
                Some(n) if n > i => {
                    push_text(&mut children, self.source[i..n].to_string(), i, n, pending.take());
                    i = n;
                }
                Some(_) => i += 1,
                None => {
                    push_text(&mut children, self.source[i..].to_string(), i, self.source.len(), pending.take());
                    break;
                }
            }
        }

        Ok((children, i))
    }
}

/// Convert a closer marker into the level string consumed by the next text node.
fn right_trim(mode: TrimMode) -> Option<&'static str> {
    match mode {
        TrimMode::None => None,
        other => Some(other.as_str()),
    }
}

/// Append a text node, applying a pending right-side trim from the previous tag.
fn push_text(children: &mut Vec<NativeNode>, mut value: String, start: usize, end: usize, pending: Option<&str>) {
    if let Some(level) = pending {
        dejavu_ir::trim::trim_prefix(&mut value, level);
    }
    children.push(NativeNode::new(NativeKind::Text(value), Some(Span { start, end })));
}

/// `else` / `else if` / `end *` closers do not own whitespace markers.
fn reject_closer_trim(
    source: &str,
    file: &str,
    tag: &TagScan,
    head: &CodeHead<'_>,
) -> Result<(), ParseError> {
    if tag.trim_left == TrimMode::None && tag.trim_right == TrimMode::None {
        return Ok(());
    }
    let label = match head {
        CodeHead::EndIf => "end if",
        CodeHead::EndLoop => "end loop",
        CodeHead::EndBlock => "end block",
        CodeHead::Else => "else",
        CodeHead::ElseIf { .. } => "else if",
        _ => "closer",
    };
    Err(ParseError::at(
        source,
        file,
        tag.inner_range.start,
        tag.inner.len().max(1),
        format!("whitespace control markers are not supported on `{label}`"),
        "unsupported marker",
    )
    .with_code(codes::PARSE_UNEXPECTED_TOKEN))
}

fn should_stop_head(head: &CodeHead<'_>, stop: &[&str]) -> bool {
    let kind = head_stop_kind(head);
    !kind.is_empty() && stop.iter().any(|s| *s == kind)
}

/// Parse source into an [`IrDocument`] using default delimiters (`<%` / `%>` / `<#` / `#>`).
pub fn parse(source: impl AsRef<str>) -> ParseResult<IrDocument> {
    parse_named(source.as_ref(), "template.dejavu")
}

/// Parse with a file name for diagnostics, using default delimiters.
pub fn parse_named(source: &str, file: &str) -> ParseResult<IrDocument> {
    let parser = Parser::new(source, file);
    finish_parse(parser)
}

/// Parse with a [`Language`] config (delimiters + feature flags).
pub fn parse_with(source: &str, file: &str, language: &Language) -> ParseResult<IrDocument> {
    let parser = Parser::with_language(source, file, language);
    finish_parse(parser)
}

fn finish_parse(parser: Parser<'_>) -> ParseResult<IrDocument> {
    let native = parser.parse_template()?;
    let mut doc = encode_native(&native);
    doc.language = default_language();
    Ok(doc)
}

/// Parse into the Rust-native T1 AST with default delimiters.
pub fn parse_native(source: impl AsRef<str>) -> ParseResult<NativeTemplate> {
    let source = source.as_ref();
    Parser::new(source, "template.dejavu").parse_template()
}

/// Parse into the Rust-native T1 AST with a [`Language`] config.
pub fn parse_native_with(source: impl AsRef<str>, language: &Language) -> ParseResult<NativeTemplate> {
    let source = source.as_ref();
    Parser::with_language(source, "template.dejavu", language).parse_template()
}
