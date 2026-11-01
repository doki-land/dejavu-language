//! Template control-flow parsing (`if` / `loop` / `block`) over classified code heads.

use dejavu_diagnostic::codes;
use dejavu_ir::{NativeExpr, NativeImportName, NativeKind, NativeNode, NativeParam, types::Span};

use super::{
    Parser, STANDALONE_RIGHT,
    expr::parse_expr,
    stmt_head::{CodeHead, classify_code},
    trim_first_text, trim_last_text,
};
use crate::{error::ParseError, lexer::TrimMode, token::CodeToken};

impl Parser<'_> {
    pub(crate) fn parse_if(
        &self,
        open: usize,
        mut i: usize,
        test_src: &str,
        test_base: usize,
        trim: TrimMode,
        trim_end: TrimMode,
        pending: Vec<&'static str>,
    ) -> Result<(NativeNode, usize, Vec<&'static str>), ParseError> {
        let test = parse_expr(self.source, self.file, test_base, test_src)?;
        let (mut consequent, i2) = self.parse_until(i, &["else if", "else", "end if"])?;
        i = i2;
        let mut else_ifs: Vec<(NativeExpr, Vec<NativeNode>)> = Vec::new();
        let mut alternate: Option<Vec<NativeNode>> = None;
        let mut closer_pending: Vec<&'static str> = Vec::new();

        loop {
            let tag = self.scan_tag(i)?;
            let trimmed = tag.inner.trim();
            let content_base = tag.inner_range.start + (tag.inner.len() - tag.inner.trim_start().len());
            let head = classify_code(self.source, self.file, trimmed, content_base)?;
            // Closers never carry markers (rejected upstream), so standalone
            // depends only on the line position.
            let (sa_left, sa_right) = self.standalone_sides(i, tag.next, TrimMode::None, TrimMode::None);
            match head {
                CodeHead::ElseIf { test_src, test_base } => {
                    if sa_left {
                        trim_branch_tail(&mut consequent, &mut else_ifs, &mut alternate);
                    }
                    i = tag.next;
                    let t = parse_expr(self.source, self.file, test_base, test_src)?;
                    let (mut body, i3) = self.parse_until(i, &["else if", "else", "end if"])?;
                    if sa_right {
                        trim_first_text(&mut body);
                    }
                    else_ifs.push((t, body));
                    i = i3;
                }
                CodeHead::Else => {
                    if sa_left {
                        trim_branch_tail(&mut consequent, &mut else_ifs, &mut alternate);
                    }
                    i = tag.next;
                    let (mut body, i3) = self.parse_until(i, &["end if"])?;
                    if sa_right {
                        trim_first_text(&mut body);
                    }
                    alternate = Some(body);
                    i = i3;
                    let end_tag = self.scan_tag(i)?;
                    let end_trim = end_tag.inner.trim();
                    let end_base = end_tag.inner_range.start + (end_tag.inner.len() - end_tag.inner.trim_start().len());
                    match classify_code(self.source, self.file, end_trim, end_base)? {
                        CodeHead::EndIf => {
                            let (e_left, e_right) = self.standalone_sides(i, end_tag.next, TrimMode::None, TrimMode::None);
                            if e_left {
                                trim_branch_tail(&mut consequent, &mut else_ifs, &mut alternate);
                            }
                            if e_right {
                                closer_pending.extend_from_slice(&STANDALONE_RIGHT);
                            }
                            i = end_tag.next;
                        }
                        _ => {
                            return Err(ParseError::at(
                                self.source,
                                self.file,
                                end_tag.inner_range.start,
                                end_tag.inner_range.len(),
                                "expected `end if`",
                                "expected end if",
                            )
                            .with_code(codes::PARSE_UNCLOSED_BLOCK));
                        }
                    }
                    break;
                }
                CodeHead::EndIf => {
                    if sa_left {
                        trim_branch_tail(&mut consequent, &mut else_ifs, &mut alternate);
                    }
                    if sa_right {
                        closer_pending.extend_from_slice(&STANDALONE_RIGHT);
                    }
                    i = tag.next;
                    break;
                }
                other => {
                    return Err(ParseError::at(
                        self.source,
                        self.file,
                        tag.inner_range.start,
                        tag.inner_range.len(),
                        format!("expected if closer, got `{other:?}`"),
                        "unexpected",
                    )
                    .with_code(codes::PARSE_UNCLOSED_BLOCK));
                }
            }
        }

        let mut kind = NativeKind::If { test, consequent, else_ifs, alternate, trim: trim.as_str().into() };
        apply_open_trim(&mut kind, &pending, trim_end);
        Ok((NativeNode::new(kind, Some(Span { start: open, end: i })), i, closer_pending))
    }

    pub(crate) fn parse_loop(
        &self,
        open: usize,
        mut i: usize,
        item: String,
        iter_src: &str,
        iter_base: usize,
        trim: TrimMode,
        trim_end: TrimMode,
        pending: Vec<&'static str>,
    ) -> Result<(NativeNode, usize, Vec<&'static str>), ParseError> {
        let iterable = parse_expr(self.source, self.file, iter_base, iter_src)?;

        let (mut body, i2) = self.parse_until(i, &["end loop"])?;
        i = i2;
        let end_tag = self.scan_tag(i)?;
        let end_trim = end_tag.inner.trim();
        let end_base = end_tag.inner_range.start + (end_tag.inner.len() - end_tag.inner.trim_start().len());
        let mut closer_pending = Vec::new();
        match classify_code(self.source, self.file, end_trim, end_base)? {
            CodeHead::EndLoop => {
                let (sa_left, sa_right) = self.standalone_sides(i, end_tag.next, TrimMode::None, TrimMode::None);
                if sa_left {
                    trim_last_text(&mut body);
                }
                if sa_right {
                    closer_pending.extend_from_slice(&STANDALONE_RIGHT);
                }
                i = end_tag.next;
            }
            _ => {
                return Err(ParseError::at(
                    self.source,
                    self.file,
                    end_tag.inner_range.start,
                    end_tag.inner_range.len(),
                    "expected `end loop`",
                    "expected end loop",
                )
                .with_code(codes::PARSE_UNCLOSED_BLOCK));
            }
        }

        let mut kind = NativeKind::For { item, iterable, body, trim: trim.as_str().into() };
        apply_open_trim(&mut kind, &pending, trim_end);
        Ok((NativeNode::new(kind, Some(Span { start: open, end: i })), i, closer_pending))
    }

    pub(crate) fn parse_partial(
        &self,
        open: usize,
        mut i: usize,
        params_src: &str,
        params_base: usize,
        trim: TrimMode,
        trim_end: TrimMode,
        pending: Vec<&'static str>,
    ) -> Result<(NativeNode, usize, Vec<&'static str>), ParseError> {
        let params = parse_params(self.source, self.file, params_src, params_base)?;
        let (mut body, i2) = self.parse_until(i, &["end partial"])?;
        i = i2;
        let end_tag = self.scan_tag(i)?;
        let end_trim = end_tag.inner.trim();
        let end_base = end_tag.inner_range.start + (end_tag.inner.len() - end_tag.inner.trim_start().len());
        let mut closer_pending = Vec::new();
        match classify_code(self.source, self.file, end_trim, end_base)? {
            CodeHead::EndPartial => {
                let (sa_left, sa_right) = self.standalone_sides(i, end_tag.next, TrimMode::None, TrimMode::None);
                if sa_left {
                    trim_last_text(&mut body);
                }
                if sa_right {
                    closer_pending.extend_from_slice(&STANDALONE_RIGHT);
                }
                i = end_tag.next;
            }
            _ => {
                return Err(ParseError::at(
                    self.source,
                    self.file,
                    end_tag.inner_range.start,
                    end_tag.inner_range.len(),
                    "expected `end partial`",
                    "expected end partial",
                )
                .with_code(codes::PARSE_UNCLOSED_BLOCK));
            }
        }

        let mut kind = NativeKind::Partial { params, body, trim: trim.as_str().into() };
        apply_open_trim(&mut kind, &pending, trim_end);
        Ok((NativeNode::new(kind, Some(Span { start: open, end: i })), i, closer_pending))
    }

    pub(crate) fn parse_import(
        &self,
        open: usize,
        i: usize,
        path: String,
        names: Vec<(String, Option<String>)>,
        trim: TrimMode,
    ) -> (NativeNode, usize) {
        let kind = NativeKind::Import {
            path,
            names: names.into_iter().map(|(name, alias)| NativeImportName { name, alias }).collect(),
            trim: trim.as_str().into(),
        };
        (NativeNode::new(kind, Some(Span { start: open, end: i })), i)
    }

    pub(crate) fn parse_macro(
        &self,
        open: usize,
        mut i: usize,
        name: String,
        params_src: &str,
        params_base: usize,
        trim: TrimMode,
        trim_end: TrimMode,
        pending: Vec<&'static str>,
    ) -> Result<(NativeNode, usize, Vec<&'static str>), ParseError> {
        let params = if params_src.trim().is_empty() {
            Vec::new()
        }
        else {
            parse_params(self.source, self.file, params_src, params_base)?
        };
        if params.iter().any(|p| p.name == "caller") {
            return Err(ParseError::at(
                self.source,
                self.file,
                params_base,
                params_src.len().max(1),
                "`caller` is reserved for the caller block and cannot be a macro parameter",
                "reserved parameter",
            )
            .with_code(codes::PARSE_UNEXPECTED_TOKEN));
        }
        let (mut body, i2) = self.parse_until(i, &["end macro"])?;
        i = i2;
        let end_tag = self.scan_tag(i)?;
        let end_trim = end_tag.inner.trim();
        let end_base = end_tag.inner_range.start + (end_tag.inner.len() - end_tag.inner.trim_start().len());
        let mut closer_pending = Vec::new();
        match classify_code(self.source, self.file, end_trim, end_base)? {
            CodeHead::EndMacro => {
                let (sa_left, sa_right) = self.standalone_sides(i, end_tag.next, TrimMode::None, TrimMode::None);
                if sa_left {
                    trim_last_text(&mut body);
                }
                if sa_right {
                    closer_pending.extend_from_slice(&STANDALONE_RIGHT);
                }
                i = end_tag.next;
            }
            _ => {
                return Err(ParseError::at(
                    self.source,
                    self.file,
                    end_tag.inner_range.start,
                    end_tag.inner_range.len(),
                    "expected `end macro`",
                    "expected end macro",
                )
                .with_code(codes::PARSE_UNCLOSED_BLOCK));
            }
        }

        let mut kind = NativeKind::Macro { name, params, body, trim: trim.as_str().into() };
        apply_open_trim(&mut kind, &pending, trim_end);
        Ok((NativeNode::new(kind, Some(Span { start: open, end: i })), i, closer_pending))
    }

    /// Parse `call name(args)` or `call(a, b) name(args)` … `end call`.
    ///
    /// The argument list is parsed as a normal bare-identifier call so arity
    /// and expression errors match inline macro calls. Caller parameters are
    /// bare identifiers without defaults.
    pub(crate) fn parse_call(
        &self,
        open: usize,
        mut i: usize,
        src: &str,
        base: usize,
        params_src: Option<(&str, usize)>,
        trim: TrimMode,
        trim_end: TrimMode,
        pending: Vec<&'static str>,
    ) -> Result<(NativeNode, usize, Vec<&'static str>), ParseError> {
        let params = match params_src {
            Some((src, base)) => parse_caller_params(self.source, self.file, src, base)?,
            None => Vec::new(),
        };
        let call_err = |message: &str| {
            ParseError::at(self.source, self.file, base, src.len().max(1), message, "expected macro call")
                .with_code(codes::PARSE_UNEXPECTED_TOKEN)
        };
        let (name, arguments) = match parse_expr(self.source, self.file, base, src)? {
            NativeExpr::Call { callee, arguments } => match *callee {
                NativeExpr::Identifier(name) => (name, arguments),
                _ => return Err(call_err("`call` requires a bare macro name")),
            },
            _ => return Err(call_err("`call` requires `name(args)`")),
        };
        let (mut body, i2) = self.parse_until(i, &["end call"])?;
        i = i2;
        if i >= self.source.len() {
            return Err(ParseError::at(
                self.source,
                self.file,
                open,
                1,
                "unclosed `call` block, expected `end call`",
                "call block opens here",
            )
            .with_code(codes::PARSE_UNCLOSED_BLOCK));
        }
        let end_tag = self.scan_tag(i)?;
        let end_trim = end_tag.inner.trim();
        let end_base = end_tag.inner_range.start + (end_tag.inner.len() - end_tag.inner.trim_start().len());
        let mut closer_pending = Vec::new();
        match classify_code(self.source, self.file, end_trim, end_base)? {
            CodeHead::EndCall => {
                let (sa_left, sa_right) = self.standalone_sides(i, end_tag.next, TrimMode::None, TrimMode::None);
                if sa_left {
                    trim_last_text(&mut body);
                }
                if sa_right {
                    closer_pending.extend_from_slice(&STANDALONE_RIGHT);
                }
                i = end_tag.next;
            }
            _ => {
                return Err(ParseError::at(
                    self.source,
                    self.file,
                    end_tag.inner_range.start,
                    end_tag.inner_range.len(),
                    "expected `end call`",
                    "expected end call",
                )
                .with_code(codes::PARSE_UNCLOSED_BLOCK));
            }
        }

        let mut kind = NativeKind::Call { name, arguments, params, body, trim: trim.as_str().into() };
        apply_open_trim(&mut kind, &pending, trim_end);
        Ok((NativeNode::new(kind, Some(Span { start: open, end: i })), i, closer_pending))
    }

    pub(crate) fn parse_block(
        &self,
        open: usize,
        mut i: usize,
        name: String,
        trim: TrimMode,
        trim_end: TrimMode,
        pending: Vec<&'static str>,
    ) -> Result<(NativeNode, usize, Vec<&'static str>), ParseError> {
        let (mut body, i2) = self.parse_until(i, &["end block"])?;
        i = i2;
        let end_tag = self.scan_tag(i)?;
        let end_trim = end_tag.inner.trim();
        let end_base = end_tag.inner_range.start + (end_tag.inner.len() - end_tag.inner.trim_start().len());
        let mut closer_pending = Vec::new();
        match classify_code(self.source, self.file, end_trim, end_base)? {
            CodeHead::EndBlock => {
                let (sa_left, sa_right) = self.standalone_sides(i, end_tag.next, TrimMode::None, TrimMode::None);
                if sa_left {
                    trim_last_text(&mut body);
                }
                if sa_right {
                    closer_pending.extend_from_slice(&STANDALONE_RIGHT);
                }
                i = end_tag.next;
            }
            _ => {
                return Err(ParseError::at(
                    self.source,
                    self.file,
                    end_tag.inner_range.start,
                    end_tag.inner_range.len(),
                    "expected `end block`",
                    "expected end block",
                )
                .with_code(codes::PARSE_UNCLOSED_BLOCK));
            }
        }

        let mut kind = NativeKind::Block { name, body, trim: trim.as_str().into() };
        apply_open_trim(&mut kind, &pending, trim_end);
        Ok((NativeNode::new(kind, Some(Span { start: open, end: i })), i, closer_pending))
    }
}

/// Standalone left of an `else` / `else if` / `end if` closer trims the tail
/// of the branch that has just been completed.
fn trim_branch_tail(
    consequent: &mut Vec<NativeNode>,
    else_ifs: &mut [(NativeExpr, Vec<NativeNode>)],
    alternate: &mut Option<Vec<NativeNode>>,
) {
    if let Some(alt) = alternate.as_mut() {
        trim_last_text(alt);
    }
    else if let Some((_, body)) = else_ifs.last_mut() {
        trim_last_text(body);
    }
    else {
        trim_last_text(consequent);
    }
}

/// Apply the right-side trim of an opening tag to the first text node the tag
/// introduces. Inherited pending trims from a previous tag are applied first.
fn apply_open_trim(kind: &mut NativeKind, pending: &[&str], trim_end: TrimMode) {
    let mut levels: Vec<&str> = pending.to_vec();
    if trim_end != TrimMode::None {
        levels.push(trim_end.as_str());
    }
    if levels.is_empty() {
        return;
    }
    let Some(text) = first_text(kind)
    else {
        return;
    };
    for level in levels {
        dejavu_ir::trim::trim_prefix(text, level);
    }
}

fn first_text(kind: &mut NativeKind) -> Option<&mut String> {
    let nodes = match kind {
        NativeKind::If { consequent, .. } => consequent,
        NativeKind::For { body, .. } => body,
        NativeKind::Block { body, .. } => body,
        NativeKind::Partial { body, .. } => body,
        NativeKind::Macro { body, .. } | NativeKind::Call { body, .. } => body,
        _ => return None,
    };
    nodes.iter_mut().find_map(|n| match &mut n.kind {
        NativeKind::Text(value) => Some(value),
        _ => None,
    })
}

/// Parse a `partial` parameter list: `name`, `name = <literal>`, comma separated.
pub(crate) fn parse_params(
    source: &str,
    file: &str,
    params_src: &str,
    params_base: usize,
) -> Result<Vec<NativeParam>, ParseError> {
    let mut tokens: Vec<(CodeToken, std::ops::Range<usize>)> = Vec::new();
    let mut lex = CodeToken::lexer(params_src);
    while let Some(tok) = lex.next() {
        let span = lex.span();
        match tok {
            Ok(CodeToken::CodeEnd) => {
                return Err(ParseError::at(
                    source,
                    file,
                    params_base + span.start,
                    span.len(),
                    "unexpected expression terminator in partial parameters",
                    "here",
                )
                .with_code(codes::PARSE_UNEXPECTED_TOKEN));
            }
            Ok(t) => tokens.push((t, span)),
            Err(()) => {
                return Err(ParseError::at(
                    source,
                    file,
                    params_base + span.start,
                    span.len().max(1),
                    "invalid token in partial parameters",
                    "bad token",
                )
                .with_code(codes::PARSE_UNEXPECTED_TOKEN));
            }
        }
    }

    // Split into top-level comma separated segments.
    let mut segments: Vec<std::ops::Range<usize>> = Vec::new();
    let mut depth = 0i32;
    let mut start = 0usize;
    for (i, (tok, _)) in tokens.iter().enumerate() {
        match tok {
            CodeToken::LParen | CodeToken::LBracket | CodeToken::LBrace => depth += 1,
            CodeToken::RParen | CodeToken::RBracket | CodeToken::RBrace => depth -= 1,
            CodeToken::Comma if depth == 0 => {
                segments.push(start..i);
                start = i + 1;
            }
            _ => {}
        }
    }
    segments.push(start..tokens.len());

    let mut params: Vec<NativeParam> = Vec::new();
    for segment in segments {
        let Some((CodeToken::Ident(name), name_span)) = tokens.get(segment.start)
        else {
            let span = tokens.get(segment.start).map(|(_, s)| s.clone()).unwrap_or(0..params_src.len().max(1));
            return Err(ParseError::at(
                source,
                file,
                params_base + span.start,
                span.len().max(1),
                "partial parameter must be an identifier",
                "expected ident",
            )
            .with_code(codes::PARSE_UNEXPECTED_TOKEN));
        };
        if params.iter().any(|p| &p.name == name) {
            return Err(ParseError::at(
                source,
                file,
                params_base + name_span.start,
                name_span.len().max(1),
                format!("duplicate partial parameter `{name}`"),
                "duplicate parameter",
            )
            .with_code(codes::PARSE_UNEXPECTED_TOKEN));
        }
        let default = if segment.len() == 1 {
            None
        }
        else {
            match tokens.get(segment.start + 1) {
                Some((CodeToken::Assign, _)) => {}
                _ => {
                    let (_, span) = &tokens[segment.start + 1];
                    return Err(ParseError::at(
                        source,
                        file,
                        params_base + span.start,
                        span.len().max(1),
                        "expected `=` in partial parameter",
                        "expected =",
                    )
                    .with_code(codes::PARSE_UNEXPECTED_TOKEN));
                }
            }
            let value_start = segment.start + 2;
            if value_start >= segment.end {
                let (_, span) = &tokens[segment.start + 1];
                return Err(ParseError::at(
                    source,
                    file,
                    params_base + span.start,
                    span.len().max(1),
                    "partial parameter default requires an expression",
                    "expected default",
                )
                .with_code(codes::PARSE_UNEXPECTED_TOKEN));
            }
            let from = tokens[value_start].1.start;
            let to = tokens[segment.end - 1].1.end;
            let expr = parse_expr(source, file, params_base + from, &params_src[from..to])?;
            if !matches!(expr, NativeExpr::Literal(_)) {
                return Err(ParseError::at(
                    source,
                    file,
                    params_base + from,
                    (to - from).max(1),
                    "partial parameter default must be a literal",
                    "not a literal",
                )
                .with_code(codes::PARSE_UNEXPECTED_TOKEN));
            }
            Some(expr)
        };
        params.push(NativeParam { name: name.clone(), default });
    }
    Ok(params)
}

/// Parse a `call(...)` parameter list: bare identifiers, no defaults.
fn parse_caller_params(source: &str, file: &str, params_src: &str, params_base: usize) -> Result<Vec<NativeParam>, ParseError> {
    let mut tokens: Vec<(CodeToken, std::ops::Range<usize>)> = Vec::new();
    let mut lex = CodeToken::lexer(params_src);
    while let Some(tok) = lex.next() {
        let span = lex.span();
        match tok {
            Ok(CodeToken::CodeEnd) => {
                return Err(ParseError::at(
                    source,
                    file,
                    params_base + span.start,
                    span.len(),
                    "unexpected expression terminator in call parameters",
                    "here",
                )
                .with_code(codes::PARSE_UNEXPECTED_TOKEN));
            }
            Ok(t) => tokens.push((t, span)),
            Err(()) => {
                return Err(ParseError::at(
                    source,
                    file,
                    params_base + span.start,
                    span.len().max(1),
                    "invalid token in call parameters",
                    "bad token",
                )
                .with_code(codes::PARSE_UNEXPECTED_TOKEN));
            }
        }
    }
    if tokens.is_empty() {
        return Ok(Vec::new());
    }

    let mut params: Vec<NativeParam> = Vec::new();
    let mut expect_name = true;
    for (tok, span) in &tokens {
        if expect_name {
            let CodeToken::Ident(name) = tok
            else {
                return Err(ParseError::at(
                    source,
                    file,
                    params_base + span.start,
                    span.len().max(1),
                    "call parameter must be an identifier",
                    "expected ident",
                )
                .with_code(codes::PARSE_UNEXPECTED_TOKEN));
            };
            if name == "caller" {
                return Err(ParseError::at(
                    source,
                    file,
                    params_base + span.start,
                    span.len().max(1),
                    "`caller` is reserved and cannot be a call parameter",
                    "reserved parameter",
                )
                .with_code(codes::PARSE_UNEXPECTED_TOKEN));
            }
            if params.iter().any(|p| &p.name == name) {
                return Err(ParseError::at(
                    source,
                    file,
                    params_base + span.start,
                    span.len().max(1),
                    format!("duplicate call parameter `{name}`"),
                    "duplicate parameter",
                )
                .with_code(codes::PARSE_UNEXPECTED_TOKEN));
            }
            params.push(NativeParam { name: name.clone(), default: None });
        }
        else if !matches!(tok, CodeToken::Comma) {
            return Err(ParseError::at(
                source,
                file,
                params_base + span.start,
                span.len().max(1),
                "call parameters must be comma separated identifiers",
                "expected `,`",
            )
            .with_code(codes::PARSE_UNEXPECTED_TOKEN));
        }
        expect_name = !expect_name;
    }
    if expect_name {
        return Err(ParseError::at(
            source,
            file,
            params_base + params_src.trim_end().len().max(1),
            1,
            "trailing comma in call parameters",
            "expected ident",
        )
        .with_code(codes::PARSE_UNEXPECTED_TOKEN));
    }
    Ok(params)
}
