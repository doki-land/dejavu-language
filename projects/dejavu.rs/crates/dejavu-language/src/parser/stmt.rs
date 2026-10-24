//! Template control-flow parsing (`if` / `loop` / `block`) over classified code heads.

use dejavu_diagnostic::codes;
use dejavu_ir::types::Span;
use dejavu_ir::{NativeKind, NativeNode};

use super::expr::parse_expr;
use super::stmt_head::{CodeHead, classify_code};
use super::Parser;
use crate::error::ParseError;
use crate::lexer::TrimMode;

impl Parser<'_> {
    pub(crate) fn parse_if(
        &self,
        open: usize,
        mut i: usize,
        test_src: &str,
        test_base: usize,
        trim: TrimMode,
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

        Ok((
            NativeNode::new(
                NativeKind::If { test, consequent, else_ifs, alternate, trim: trim.as_str().into() },
                Some(Span { start: open, end: i }),
            ),
            i,
        ))
    }

    pub(crate) fn parse_loop(
        &self,
        open: usize,
        mut i: usize,
        item: String,
        iter_src: &str,
        iter_base: usize,
        trim: TrimMode,
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

        Ok((
            NativeNode::new(
                NativeKind::For { item, index: None, iterable, body, trim: trim.as_str().into() },
                Some(Span { start: open, end: i }),
            ),
            i,
        ))
    }

    pub(crate) fn parse_block(
        &self,
        open: usize,
        mut i: usize,
        name: String,
        trim: TrimMode,
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

        Ok((
            NativeNode::new(
                NativeKind::Block { name, body, trim: trim.as_str().into() },
                Some(Span { start: open, end: i }),
            ),
            i,
        ))
    }
}
