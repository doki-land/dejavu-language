//! Template control-flow parsing (`if` / `loop` / `block`) over classified code heads.

use dejavu_diagnostic::codes;
use dejavu_ir::types::Span;
use dejavu_ir::{NativeExpr, NativeKind, NativeNode, NativeParam};

use super::expr::parse_expr;
use super::stmt_head::{CodeHead, classify_code};
use super::Parser;
use crate::error::ParseError;
use crate::lexer::TrimMode;
use crate::token::CodeToken;

impl Parser<'_> {
    pub(crate) fn parse_if(
        &self,
        open: usize,
        mut i: usize,
        test_src: &str,
        test_base: usize,
        trim: TrimMode,
        trim_end: TrimMode,
        pending: Option<&str>,
    ) -> Result<(NativeNode, usize), ParseError> {
        let test = parse_expr(self.source, self.file, test_base, test_src)?;
        let (consequent, i2) = self.parse_until(i, &["else if", "else", "end if"])?;
        i = i2;
        let mut else_ifs = Vec::new();
        let mut alternate = None;

        loop {
            let (code, code_span, next_i) = self.peek_code(i)?;
            let trimmed = code.trim();
            let content_base = code_span.start + (code.len() - code.trim_start().len());
            let head = classify_code(self.source, self.file, trimmed, content_base)?;
            match head {
                CodeHead::ElseIf { test_src, test_base } => {
                    i = next_i;
                    let t = parse_expr(self.source, self.file, test_base, test_src)?;
                    let (body, i3) = self.parse_until(i, &["else if", "else", "end if"])?;
                    else_ifs.push((t, body));
                    i = i3;
                }
                CodeHead::Else => {
                    i = next_i;
                    let (body, i3) = self.parse_until(i, &["end if"])?;
                    alternate = Some(body);
                    i = i3;
                    let (end_code, end_span, end_i) = self.peek_code(i)?;
                    let end_trim = end_code.trim();
                    let end_base = end_span.start + (end_code.len() - end_code.trim_start().len());
                    match classify_code(self.source, self.file, end_trim, end_base)? {
                        CodeHead::EndIf => i = end_i,
                        _ => {
                            return Err(ParseError::at(
                                self.source,
                                self.file,
                                end_span.start,
                                end_span.len(),
                                "expected `end if`",
                                "expected end if",
                            )
                            .with_code(codes::PARSE_UNCLOSED_BLOCK));
                        }
                    }
                    break;
                }
                CodeHead::EndIf => {
                    i = next_i;
                    break;
                }
                other => {
                    return Err(ParseError::at(
                        self.source,
                        self.file,
                        code_span.start,
                        code_span.len(),
                        format!("expected if closer, got `{other:?}`"),
                        "unexpected",
                    )
                    .with_code(codes::PARSE_UNCLOSED_BLOCK));
                }
            }
        }

        let mut kind = NativeKind::If { test, consequent, else_ifs, alternate, trim: trim.as_str().into() };
        apply_open_trim(&mut kind, pending, trim_end);
        Ok((NativeNode::new(kind, Some(Span { start: open, end: i })), i))
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
        pending: Option<&str>,
    ) -> Result<(NativeNode, usize), ParseError> {
        let iterable = parse_expr(self.source, self.file, iter_base, iter_src)?;

        let (body, i2) = self.parse_until(i, &["end loop"])?;
        i = i2;
        let (end_code, end_span, end_i) = self.peek_code(i)?;
        let end_trim = end_code.trim();
        let end_base = end_span.start + (end_code.len() - end_code.trim_start().len());
        match classify_code(self.source, self.file, end_trim, end_base)? {
            CodeHead::EndLoop => i = end_i,
            _ => {
                return Err(ParseError::at(
                    self.source,
                    self.file,
                    end_span.start,
                    end_span.len(),
                    "expected `end loop`",
                    "expected end loop",
                )
                .with_code(codes::PARSE_UNCLOSED_BLOCK));
            }
        }

        let mut kind = NativeKind::For { item, index: None, iterable, body, trim: trim.as_str().into() };
        apply_open_trim(&mut kind, pending, trim_end);
        Ok((NativeNode::new(kind, Some(Span { start: open, end: i })), i))
    }

    pub(crate) fn parse_partial(
        &self,
        open: usize,
        mut i: usize,
        params_src: &str,
        params_base: usize,
        trim: TrimMode,
        trim_end: TrimMode,
        pending: Option<&str>,
    ) -> Result<(NativeNode, usize), ParseError> {
        let params = parse_params(self.source, self.file, params_src, params_base)?;
        let (body, i2) = self.parse_until(i, &["end partial"])?;
        i = i2;
        let (end_code, end_span, end_i) = self.peek_code(i)?;
        let end_trim = end_code.trim();
        let end_base = end_span.start + (end_code.len() - end_code.trim_start().len());
        match classify_code(self.source, self.file, end_trim, end_base)? {
            CodeHead::EndPartial => i = end_i,
            _ => {
                return Err(ParseError::at(
                    self.source,
                    self.file,
                    end_span.start,
                    end_span.len(),
                    "expected `end partial`",
                    "expected end partial",
                )
                .with_code(codes::PARSE_UNCLOSED_BLOCK));
            }
        }

        let mut kind = NativeKind::Partial { params, body, trim: trim.as_str().into() };
        apply_open_trim(&mut kind, pending, trim_end);
        Ok((NativeNode::new(kind, Some(Span { start: open, end: i })), i))
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
        pending: Option<&str>,
    ) -> Result<(NativeNode, usize), ParseError> {
        let params = if params_src.trim().is_empty() {
            Vec::new()
        } else {
            parse_params(self.source, self.file, params_src, params_base)?
        };
        let (body, i2) = self.parse_until(i, &["end macro"])?;
        i = i2;
        let (end_code, end_span, end_i) = self.peek_code(i)?;
        let end_trim = end_code.trim();
        let end_base = end_span.start + (end_code.len() - end_code.trim_start().len());
        match classify_code(self.source, self.file, end_trim, end_base)? {
            CodeHead::EndMacro => i = end_i,
            _ => {
                return Err(ParseError::at(
                    self.source,
                    self.file,
                    end_span.start,
                    end_span.len(),
                    "expected `end macro`",
                    "expected end macro",
                )
                .with_code(codes::PARSE_UNCLOSED_BLOCK));
            }
        }

        let mut kind = NativeKind::Macro { name, params, body, trim: trim.as_str().into() };
        apply_open_trim(&mut kind, pending, trim_end);
        Ok((NativeNode::new(kind, Some(Span { start: open, end: i })), i))
    }

    pub(crate) fn parse_block(
        &self,
        open: usize,
        mut i: usize,
        name: String,
        trim: TrimMode,
        trim_end: TrimMode,
        pending: Option<&str>,
    ) -> Result<(NativeNode, usize), ParseError> {
        let (body, i2) = self.parse_until(i, &["end block"])?;
        i = i2;
        let (end_code, end_span, end_i) = self.peek_code(i)?;
        let end_trim = end_code.trim();
        let end_base = end_span.start + (end_code.len() - end_code.trim_start().len());
        match classify_code(self.source, self.file, end_trim, end_base)? {
            CodeHead::EndBlock => i = end_i,
            _ => {
                return Err(ParseError::at(
                    self.source,
                    self.file,
                    end_span.start,
                    end_span.len(),
                    "expected `end block`",
                    "expected end block",
                )
                .with_code(codes::PARSE_UNCLOSED_BLOCK));
            }
        }

        let mut kind = NativeKind::Block { name, body, trim: trim.as_str().into() };
        apply_open_trim(&mut kind, pending, trim_end);
        Ok((NativeNode::new(kind, Some(Span { start: open, end: i })), i))
    }
}

/// Apply the right-side trim of an opening tag to the first text node the tag
/// introduces. Inherited pending trims from a previous tag are applied first.
fn apply_open_trim(kind: &mut NativeKind, pending: Option<&str>, trim_end: TrimMode) {
    let levels: Vec<&str> = pending
        .into_iter()
        .chain(match trim_end {
            TrimMode::None => None,
            other => Some(other.as_str()),
        })
        .collect();
    if levels.is_empty() {
        return;
    }
    let Some(text) = first_text(kind) else {
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
        NativeKind::Macro { body, .. } => body,
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
        let Some((CodeToken::Ident(name), name_span)) = tokens.get(segment.start) else {
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
        } else {
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
