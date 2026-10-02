//! Hand-written recursive-descent template parser.

mod expr;
mod stmt;
mod stmt_head;

use dejavu_ir::{IrDocument, NativeNode, NativeTemplate, encode_native};
use expr::parse_expr;
use stmt_head::{CodeHead, classify_code, head_stop_kind};

use crate::error::{ParseError, ParseResult};
use crate::lexer::TrimMode;

pub struct Parser<'a> {
    pub(crate) source: &'a str,
    pub(crate) file: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str, file: &'a str) -> Self {
        Self { source, file }
    }

    pub fn parse_template(&self) -> Result<NativeTemplate, ParseError> {
        let (children, _) = self.parse_body(0, &[])?;
        Ok(NativeTemplate { children })
    }

    fn parse_body(&self, mut i: usize, stop: &[&str]) -> Result<(Vec<NativeNode>, usize), ParseError> {
        let mut children = Vec::new();
        let bytes = self.source.as_bytes();

        while i < self.source.len() {
            if self.source[i..].starts_with("<%") && !stop.is_empty() {
                let (code, code_span, _) = peek_code(self.source, self.file, i)?;
                let trimmed = code.trim();
                let content_base = code_span.start + (code.len() - code.trim_start().len());
                let head = classify_code(self.source, self.file, trimmed, content_base)?;
                if should_stop_head(&head, stop) {
                    return Ok((children, i));
                }
            }

            if self.source[i..].starts_with("<#") {
                if let Some(end) = self.source[i + 2..].find("#>") {
                    let inner = &self.source[i + 2..i + 2 + end];
                    children.push(NativeNode::Comment(inner.to_string()));
                    i = i + 2 + end + 2;
                    continue;
                }
                return Err(ParseError::at(self.source, self.file, i, 2, "unclosed comment", "comment starts here"));
            }

            if self.source[i..].starts_with("<%") {
                let open = i;
                let mut j = i + 2;
                let mut trim = TrimMode::None;
                if j < self.source.len() {
                    if let Some(t) = TrimMode::from_byte(bytes[j]) {
                        trim = t;
                        j += 1;
                    } else if bytes[j] == b'!' {
                        children.push(NativeNode::Text("<%".into()));
                        i += 3;
                        continue;
                    }
                }
                let rest = &self.source[j..];
                let end = rest
                    .find("%>")
                    .ok_or_else(|| ParseError::at(self.source, self.file, open, 2, "unclosed code block", "opens here"))?;
                let code_raw = &rest[..end];
                let code = code_raw.trim();
                let code_content_start = j + (code_raw.len() - code_raw.trim_start().len());
                let next_i = j + end + 2;
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
                    CodeHead::EndIf | CodeHead::EndLoop | CodeHead::Else | CodeHead::ElseIf { .. } => {
                        if stop.is_empty() {
                            let label = match &head {
                                CodeHead::EndIf => "end if",
                                CodeHead::EndLoop => "end loop",
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
                            ));
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

            let next_code = self.source[i..].find("<%").map(|p| i + p);
            let next_comment = self.source[i..].find("<#").map(|p| i + p);
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

pub(crate) fn parse_until(parser: &Parser<'_>, i: usize, stop: &[&str]) -> Result<(Vec<NativeNode>, usize), ParseError> {
    parser.parse_body(i, stop)
}

/// Returns `(code_inner, content_span_in_source, index_after_close)`.
pub(crate) fn peek_code<'a>(
    source: &'a str,
    file: &str,
    i: usize,
) -> Result<(String, std::ops::Range<usize>, usize), ParseError> {
    if !source[i..].starts_with("<%") {
        return Err(ParseError::at(source, file, i, 1, "expected code open `<%`", "here"));
    }
    let mut j = i + 2;
    let b = source.as_bytes();
    if matches!(b.get(j), Some(b'.' | b'_' | b'-' | b'~' | b'=')) {
        j += 1;
    }
    let rest = &source[j..];
    let end = rest.find("%>").ok_or_else(|| ParseError::at(source, file, i, 2, "unclosed code block", "opens here"))?;
    let inner = rest[..end].to_string();
    let content_start = j;
    Ok((inner, content_start..content_start + end, j + end + 2))
}

/// Parse source into an [`IrDocument`].
pub fn parse(source: impl AsRef<str>) -> ParseResult<IrDocument> {
    parse_named(source.as_ref(), "template.dejavu")
}

/// Parse with a file name for diagnostics.
pub fn parse_named(source: &str, file: &str) -> ParseResult<IrDocument> {
    let parser = Parser::new(source, file);
    let native = parser.parse_template().map_err(miette::Error::from)?;
    let mut doc = encode_native(&native);
    doc.language = dejavu_ir::default_language();
    Ok(doc)
}

/// Parse into the Rust-native T1 AST.
pub fn parse_native(source: impl AsRef<str>) -> ParseResult<NativeTemplate> {
    let source = source.as_ref();
    Parser::new(source, "template.dejavu").parse_template().map_err(miette::Error::from)
}
