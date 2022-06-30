//! Classify `<% ... %>` heads from a [`CodeToken`] stream.

use crate::error::ParseError;
use crate::token::CodeToken;

#[derive(Debug)]
pub(crate) enum CodeHead<'a> {
    If { test_src: &'a str, test_base: usize },
    Loop { item: String, iter_src: &'a str, iter_base: usize },
    ElseIf { test_src: &'a str, test_base: usize },
    Else,
    EndIf,
    EndLoop,
    Expr { src: &'a str, base: usize },
}

#[derive(Clone)]
struct Spanned {
    tok: CodeToken,
    start: usize,
    end: usize,
}

/// `content` is the trimmed code interior; `content_base` is its absolute offset in source.
pub(crate) fn classify_code<'a>(
    source: &str,
    file: &str,
    content: &'a str,
    content_base: usize,
) -> Result<CodeHead<'a>, ParseError> {
    let mut spans = Vec::new();
    let mut lex = CodeToken::lexer(content);
    while let Some(tok) = lex.next() {
        let span = lex.span();
        match tok {
            Ok(CodeToken::CodeEnd) => {
                return Err(ParseError::at(
                    source,
                    file,
                    content_base + span.start,
                    span.len(),
                    "unexpected `%>` inside code",
                    "here",
                ));
            }
            Ok(t) => spans.push(Spanned { tok: t, start: span.start, end: span.end }),
            Err(()) => {
                return Err(ParseError::at(
                    source,
                    file,
                    content_base + span.start,
                    span.len().max(1),
                    "invalid token in code",
                    "bad token",
                ));
            }
        }
    }

    if spans.is_empty() {
        return Ok(CodeHead::Expr { src: content, base: content_base });
    }

    let slice = |from: usize| -> (&'a str, usize) {
        if from >= spans.len() {
            return ("", content_base + content.len());
        }
        let start = spans[from].start;
        let end = spans.last().map(|s| s.end).unwrap_or(start);
        (&content[start..end], content_base + start)
    };

    match &spans[0].tok {
        CodeToken::Ident(name) if name == "if" => {
            let (test_src, test_base) = slice(1);
            Ok(CodeHead::If { test_src, test_base })
        }
        CodeToken::Ident(name) if name == "loop" => {
            if spans.len() < 3 {
                return Err(ParseError::at(
                    source,
                    file,
                    content_base + spans[0].start,
                    (spans.last().map(|s| s.end).unwrap_or(spans[0].end) - spans[0].start).max(1),
                    "loop requires `item in iterable`",
                    "expected loop",
                ));
            }
            let CodeToken::Ident(item) = &spans[1].tok else {
                return Err(ParseError::at(
                    source,
                    file,
                    content_base + spans[1].start,
                    (spans[1].end - spans[1].start).max(1),
                    "loop requires item identifier",
                    "expected ident",
                ));
            };
            if !matches!(spans[2].tok, CodeToken::In) {
                return Err(ParseError::at(
                    source,
                    file,
                    content_base + spans[0].start,
                    (spans.last().map(|s| s.end).unwrap_or(spans[0].end) - spans[0].start).max(1),
                    "loop requires `in`",
                    "expected `item in iterable`",
                ));
            }
            let (iter_src, iter_base) = slice(3);
            Ok(CodeHead::Loop { item: item.clone(), iter_src, iter_base })
        }
        CodeToken::Ident(name) if name == "else" => {
            if spans.len() == 1 {
                return Ok(CodeHead::Else);
            }
            if let CodeToken::Ident(n) = &spans[1].tok {
                if n == "if" {
                    let (test_src, test_base) = slice(2);
                    return Ok(CodeHead::ElseIf { test_src, test_base });
                }
            }
            Err(ParseError::at(source, file, content_base + spans[1].start, 1, "unexpected tokens after `else`", "unexpected"))
        }
        CodeToken::Ident(name) if name == "end" => {
            if spans.len() == 2 {
                if let CodeToken::Ident(n) = &spans[1].tok {
                    if n == "if" {
                        return Ok(CodeHead::EndIf);
                    }
                    if n == "loop" {
                        return Ok(CodeHead::EndLoop);
                    }
                }
            }
            Err(ParseError::at(
                source,
                file,
                content_base + spans[0].start,
                (spans.last().map(|s| s.end).unwrap_or(spans[0].end) - spans[0].start).max(1),
                "expected `end if` or `end loop`",
                "unexpected",
            ))
        }
        _ => {
            let (src, base) = slice(0);
            Ok(CodeHead::Expr { src, base })
        }
    }
}

pub(crate) fn head_stop_kind(head: &CodeHead<'_>) -> &'static str {
    match head {
        CodeHead::ElseIf { .. } => "else if",
        CodeHead::Else => "else",
        CodeHead::EndIf => "end if",
        CodeHead::EndLoop => "end loop",
        _ => "",
    }
}
