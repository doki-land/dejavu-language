//! Hand-written recursive-descent template parser.

mod expr;
mod stmt;
mod stmt_head;

use dejavu_diagnostic::codes;
use dejavu_ir::{IrDocument, Language, NativeNode, NativeTemplate, TemplateConfig, default_language, encode_native};
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

impl<'a> Parser<'a> {
    pub fn new(source: &'a str, file: &'a str) -> Self {
        Self::with_language(source, file, &default_language())
    }

    pub fn with_language(source: &'a str, file: &'a str, language: &Language) -> Self {
        Self { source, file, delimiters: (&language.template).into() }
    }

    pub fn parse_template(&self) -> Result<NativeTemplate, ParseError> {
        let (children, _) = self.parse_body(0, &[])?;
        Ok(NativeTemplate { children })
    }

    /// Peek at a code block starting at index `i`.
    /// Returns `(code_inner, content_span_in_source, index_after_close)`.
    /// Uses configured delimiters.
    pub(crate) fn peek_code(&self, i: usize) -> Result<(String, std::ops::Range<usize>, usize), ParseError> {
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
        if j < self.source.len() && matches!(b[j], b'.' | b'_' | b'-' | b'~' | b'=') {
            j += 1;
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
        let inner = rest[..end].to_string();
        let content_start = j;
        Ok((inner, content_start..content_start + end, j + end + end_len))
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
        let ce_len = ds.code_end_len();
        let cms_len = ds.comment_start_len();
        let cme_len = ds.comment_end_len();
        let cs = &ds.code_start;
        let ce = &ds.code_end;
        let cms = &ds.comment_start;
        let cme = &ds.comment_end;

        while i < self.source.len() {
            if self.source[i..].starts_with(cs) && !stop.is_empty() {
                let (code, code_span, _) = self.peek_code(i)?;
                let trimmed = code.trim();
                let content_base = code_span.start + (code.len() - code.trim_start().len());
                let head = classify_code(self.source, self.file, trimmed, content_base)?;
                if should_stop_head(&head, stop) {
                    return Ok((children, i));
                }
            }

            if self.source[i..].starts_with(cms) {
                if let Some(end) = self.source[i + cms_len..].find(cme) {
                    let inner = &self.source[i + cms_len..i + cms_len + end];
                    children.push(NativeNode::Comment(inner.to_string()));
                    i = i + cms_len + end + cme_len;
                    continue;
                }
                return Err(ParseError::at(self.source, self.file, i, cms_len, "unclosed comment", "comment starts here")
                    .with_code(codes::PARSE_UNCLOSED_BLOCK));
            }

            if self.source[i..].starts_with(cs) {
                let open = i;
                let mut j = i + cs_len;
                let mut trim = TrimMode::None;
                if j < self.source.len() {
                    if let Some(t) = TrimMode::from_byte(bytes[j]) {
                        trim = t;
                        j += 1;
                    } else if bytes[j] == b'!' {
                        children.push(NativeNode::Text(cs.clone()));
                        i += cs_len + 1;
                        continue;
                    }
                }
                let rest = &self.source[j..];
                let end = rest
                    .find(ce)
                    .ok_or_else(|| {
                        ParseError::at(self.source, self.file, open, cs_len, "unclosed code block", "opens here")
                            .with_code(codes::PARSE_UNCLOSED_BLOCK)
                    })?;
                let code_raw = &rest[..end];
                let code = code_raw.trim();
                let code_content_start = j + (code_raw.len() - code_raw.trim_start().len());
                let next_i = j + end + ce_len;
                let head = classify_code(self.source, self.file, code, code_content_start)?;

                if !stop.is_empty() && should_stop_head(&head, stop) {
                    return Ok((children, open));
                }

                match head {
                    CodeHead::If { test_src, test_base } => {
                        let (node, new_i) = self.parse_if(next_i, test_src, test_base, trim)?;
                        children.push(node);
                        i = new_i;
                    }
                    CodeHead::Loop { item, iter_src, iter_base } => {
                        let (node, new_i) = self.parse_loop(next_i, item, iter_src, iter_base, trim)?;
                        children.push(node);
                        i = new_i;
                    }
                    CodeHead::Extends { parent_src, parent_base } => {
                        let parent = parse_expr(self.source, self.file, parent_base, parent_src)?;
                        children.push(NativeNode::Extends { parent, trim: trim.as_str().into() });
                        i = next_i;
                    }
                    CodeHead::Include { path_src, path_base } => {
                        let path = parse_expr(self.source, self.file, path_base, path_src)?;
                        children.push(NativeNode::Include { path, trim: trim.as_str().into() });
                        i = next_i;
                    }
                    CodeHead::Block { name } => {
                        let (node, new_i) = self.parse_block(next_i, name, trim)?;
                        children.push(node);
                        i = new_i;
                    }
                    CodeHead::Super => {
                        children.push(NativeNode::Super { trim: trim.as_str().into() });
                        i = next_i;
                    }
                    CodeHead::EndIf | CodeHead::EndLoop | CodeHead::EndBlock | CodeHead::Else | CodeHead::ElseIf { .. } => {
                        if stop.is_empty() {
                            let label = match &head {
                                CodeHead::EndIf => "end if",
                                CodeHead::EndLoop => "end loop",
                                CodeHead::EndBlock => "end block",
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
                        children.push(NativeNode::Interpolation { expression: expr, trim: trim.as_str().into(), raw: false });
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
                    children.push(NativeNode::Text(self.source[i..n].to_string()));
                    i = n;
                }
                Some(_) => i += 1,
                None => {
                    children.push(NativeNode::Text(self.source[i..].to_string()));
                    break;
                }
            }
        }

        Ok((children, i))
    }
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
