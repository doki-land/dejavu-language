//! Classify `<% ... %>` heads from a [`CodeToken`] stream.

use dejavu_diagnostic::codes;
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
    EndBlock,
    Super,
    Block { name: String },
    Partial { params_src: &'a str, params_base: usize },
    EndPartial,
    Macro { name: String, params_src: &'a str, params_base: usize },
    EndMacro,
    /// `call name(args)` … `end call`
    Call { src: &'a str, base: usize },
    EndCall,
    /// `from "<path>" import a, b as c`
    From { path: String, names: Vec<(String, Option<String>)> },
    Extends { parent_src: &'a str, parent_base: usize },
    Include { path_src: &'a str, path_base: usize, with_src: Option<&'a str>, with_base: usize, only: bool },
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
                )
                .with_code(codes::PARSE_UNEXPECTED_TOKEN));
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
                )
                .with_code(codes::PARSE_UNEXPECTED_TOKEN));
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
                )
                .with_code(codes::PARSE_UNEXPECTED_TOKEN));
            }
            let CodeToken::Ident(item) = &spans[1].tok else {
                return Err(ParseError::at(
                    source,
                    file,
                    content_base + spans[1].start,
                    (spans[1].end - spans[1].start).max(1),
                    "loop requires item identifier",
                    "expected ident",
                )
                .with_code(codes::PARSE_UNEXPECTED_TOKEN));
            };
            if !matches!(spans[2].tok, CodeToken::In) {
                return Err(ParseError::at(
                    source,
                    file,
                    content_base + spans[0].start,
                    (spans.last().map(|s| s.end).unwrap_or(spans[0].end) - spans[0].start).max(1),
                    "loop requires `in`",
                    "expected `item in iterable`",
                )
                .with_code(codes::PARSE_UNEXPECTED_TOKEN));
            }
            let (iter_src, iter_base) = slice(3);
            Ok(CodeHead::Loop { item: item.clone(), iter_src, iter_base })
        }
        CodeToken::Ident(name) if name == "extends" => {
            let (parent_src, parent_base) = slice(1);
            if parent_src.is_empty() {
                return Err(ParseError::at(
                    source,
                    file,
                    content_base + spans[0].start,
                    (spans[0].end - spans[0].start).max(1),
                    "extends requires a parent template expression",
                    "expected parent",
                )
                .with_code(codes::PARSE_UNEXPECTED_TOKEN));
            }
            Ok(CodeHead::Extends { parent_src, parent_base })
        }
        CodeToken::Ident(name) if name == "include" => {
            classify_include(source, file, content, content_base, &spans)
        }
        CodeToken::Ident(name) if name == "partial" => {
            let (params_src, params_base) = slice(1);
            if params_src.is_empty() {
                return Err(ParseError::at(
                    source,
                    file,
                    content_base + spans[0].start,
                    (spans[0].end - spans[0].start).max(1),
                    "partial requires at least one parameter",
                    "expected parameter",
                )
                .with_code(codes::PARSE_UNEXPECTED_TOKEN));
            }
            Ok(CodeHead::Partial { params_src, params_base })
        }
        CodeToken::Ident(name) if name == "from" => {
            let bad = |message: &str, label: &str, idx: usize| {
                let span = spans.get(idx).map(|s| (s.start, (s.end - s.start).max(1))).unwrap_or((spans[0].start, 1));
                ParseError::at(source, file, content_base + span.0, span.1, message, label)
                    .with_code(codes::PARSE_UNEXPECTED_TOKEN)
            };
            let CodeToken::String(path) = &spans.get(1).map(|s| &s.tok).ok_or_else(|| {
                bad("`from` requires a quoted template path", "expected string literal", 0)
            })? else {
                return Err(bad("`from` requires a quoted template path", "expected string literal", 1));
            };
            if !matches!(spans.get(2).map(|s| &s.tok), Some(CodeToken::Ident(k)) if k == "import") {
                return Err(bad("expected `import` after the template path", "expected `import`", 2));
            }
            let mut names: Vec<(String, Option<String>)> = Vec::new();
            let mut i = 3;
            while i < spans.len() {
                let CodeToken::Ident(local) = &spans[i].tok else {
                    return Err(bad("expected an imported macro name", "expected ident", i));
                };
                let mut alias = None;
                if let Some(CodeToken::Ident(k)) = spans.get(i + 1).map(|s| &s.tok) {
                    if k == "as" {
                        let CodeToken::Ident(target) = &spans.get(i + 2).map(|s| &s.tok).ok_or_else(|| {
                            bad("`as` requires a local name", "expected ident", i + 1)
                        })? else {
                            return Err(bad("`as` requires a local name", "expected ident", i + 2));
                        };
                        alias = Some(target.clone());
                        i += 3;
                    } else {
                        i += 1;
                    }
                } else {
                    i += 1;
                }
                names.push((local.clone(), alias));
                if i >= spans.len() {
                    break;
                }
                if !matches!(spans[i].tok, CodeToken::Comma) {
                    return Err(bad("imported names must be comma separated", "expected `,`", i));
                }
                i += 1;
                if i >= spans.len() {
                    return Err(bad("trailing comma in import list", "expected ident", i - 1));
                }
            }
            if names.is_empty() {
                return Err(bad("`from` requires at least one imported name", "expected ident", 2));
            }
            Ok(CodeHead::From { path: path.clone(), names })
        }
        CodeToken::Ident(name) if name == "macro" => {
            let macro_diag = |message: &str, label: &str, idx: usize| {
                let span = spans.get(idx).map(|s| (s.start, (s.end - s.start).max(1))).unwrap_or((spans[0].start, 1));
                ParseError::at(source, file, content_base + span.0, span.1, message, label)
                    .with_code(codes::PARSE_UNEXPECTED_TOKEN)
            };
            if spans.len() < 3 {
                return Err(macro_diag("macro requires a name and a parameter list", "expected `macro name(...)`", 0));
            }
            let CodeToken::Ident(macro_name) = &spans[1].tok else {
                return Err(macro_diag("macro requires a name identifier", "expected ident", 1));
            };
            if !matches!(spans[2].tok, CodeToken::LParen) {
                return Err(macro_diag("macro parameters must be written in parentheses", "expected `(`", 2));
            }
            if !matches!(spans.last().map(|s| &s.tok), Some(CodeToken::RParen)) {
                return Err(macro_diag(
                    "macro parameter list must end with `)`",
                    "expected `)`",
                    spans.len() - 1,
                ));
            }
            let open = &spans[2];
            let close = spans.last().expect("checked above");
            let params_src = &content[open.end..close.start];
            Ok(CodeHead::Macro {
                name: macro_name.clone(),
                params_src,
                params_base: content_base + open.end,
            })
        }
        CodeToken::Ident(name) if name == "call" => {
            let (src, base) = slice(1);
            if src.is_empty() {
                return Err(ParseError::at(
                    source,
                    file,
                    content_base + spans[0].start,
                    (spans[0].end - spans[0].start).max(1),
                    "call requires `macro(args)`",
                    "expected `call name(...)`",
                )
                .with_code(codes::PARSE_UNEXPECTED_TOKEN));
            }
            Ok(CodeHead::Call { src, base })
        }
        CodeToken::Ident(name) if name == "block" => {
            if spans.len() != 2 {
                return Err(ParseError::at(
                    source,
                    file,
                    content_base + spans[0].start,
                    (spans.last().map(|s| s.end).unwrap_or(spans[0].end) - spans[0].start).max(1),
                    "block requires exactly one name identifier",
                    "expected `block name`",
                )
                .with_code(codes::PARSE_UNEXPECTED_TOKEN));
            }
            let CodeToken::Ident(block_name) = &spans[1].tok else {
                return Err(ParseError::at(
                    source,
                    file,
                    content_base + spans[1].start,
                    (spans[1].end - spans[1].start).max(1),
                    "block requires a name identifier",
                    "expected ident",
                )
                .with_code(codes::PARSE_UNEXPECTED_TOKEN));
            };
            Ok(CodeHead::Block { name: block_name.clone() })
        }
        CodeToken::Ident(name) if name == "super" => {
            if spans.len() != 1 {
                return Err(ParseError::at(
                    source,
                    file,
                    content_base + spans[1].start,
                    1,
                    "unexpected tokens after `super`",
                    "unexpected",
                )
                .with_code(codes::PARSE_UNEXPECTED_TOKEN));
            }
            Ok(CodeHead::Super)
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
            Err(ParseError::at(source, file, content_base + spans[1].start, 1, "unexpected tokens after `else`", "unexpected")
                .with_code(codes::PARSE_UNEXPECTED_TOKEN))
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
                    if n == "block" {
                        return Ok(CodeHead::EndBlock);
                    }
                    if n == "partial" {
                        return Ok(CodeHead::EndPartial);
                    }
                    if n == "macro" {
                        return Ok(CodeHead::EndMacro);
                    }
                    if n == "call" {
                        return Ok(CodeHead::EndCall);
                    }
                }
            }
            Err(ParseError::at(
                source,
                file,
                content_base + spans[0].start,
                (spans.last().map(|s| s.end).unwrap_or(spans[0].end) - spans[0].start).max(1),
                "expected `end if`, `end loop`, `end block`, `end partial`, or `end macro`",
                "unexpected",
            )
            .with_code(codes::PARSE_UNEXPECTED_TOKEN))
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
        CodeHead::EndBlock => "end block",
        CodeHead::EndPartial => "end partial",
        CodeHead::EndMacro => "end macro",
        CodeHead::EndCall => "end call",
        _ => "",
    }
}

/// Sub-keyword of `include` that starts a context modifier.
fn include_modifier(tok: &CodeToken) -> Option<&'static str> {
    match tok {
        CodeToken::Ident(name) if name == "with" => Some("with"),
        CodeToken::Ident(name) if name == "only" => Some("only"),
        CodeToken::Ident(name) if name == "without" => Some("without"),
        _ => None,
    }
}

/// Classify `include <path> [with <expr>] [only | without context]`.
///
/// Context modifiers are language keywords, so they are located at bracket
/// depth zero after the path expression. Semantics:
/// - `include path` — child inherits the caller context.
/// - `include path with expr` — child inherits the caller context plus `expr`.
/// - `include path only` / `without context` — child sees no caller context.
/// - `include path with expr only` — child sees only `expr`.
fn classify_include<'a>(
    source: &str,
    file: &str,
    content: &'a str,
    content_base: usize,
    spans: &[Spanned],
) -> Result<CodeHead<'a>, ParseError> {
    let err = |idx: usize, message: &str, label: &str| {
        ParseError::at(
            source,
            file,
            content_base + spans[idx].start,
            (spans[idx].end - spans[idx].start).max(1),
            message,
            label,
        )
        .with_code(codes::PARSE_UNEXPECTED_TOKEN)
    };
    let slice = |from: usize, to: usize| -> (&'a str, usize) {
        if from >= to {
            return ("", content_base + content.len());
        }
        let start = spans[from].start;
        let end = spans[to - 1].end;
        (&content[start..end], content_base + start)
    };

    let modifier_at = |from: usize| -> Option<usize> {
        let mut depth = 0i32;
        for i in from..spans.len() {
            match spans[i].tok {
                CodeToken::LParen | CodeToken::LBracket => depth += 1,
                CodeToken::RParen | CodeToken::RBracket => depth -= 1,
                _ => {}
            }
            if depth == 0 && include_modifier(&spans[i].tok).is_some() {
                return Some(i);
            }
        }
        None
    };

    let path_end = modifier_at(1).unwrap_or(spans.len());
    let (path_src, path_base) = slice(1, path_end);
    if path_src.is_empty() {
        return Err(err(0, "include requires a path expression", "expected path"));
    }

    let mut with_src = None;
    let mut with_base = content_base + content.len();
    let mut only = false;
    let mut cursor = path_end;
    while cursor < spans.len() {
        match include_modifier(&spans[cursor].tok) {
            Some("with") => {
                if with_src.is_some() {
                    return Err(err(cursor, "include accepts a single `with` expression", "duplicate with"));
                }
                let next = cursor + 1;
                let end = modifier_at(next).unwrap_or(spans.len());
                let (src, base) = slice(next, end);
                if src.is_empty() {
                    return Err(err(cursor, "`with` requires a context expression", "expected expression"));
                }
                with_src = Some(src);
                with_base = base;
                cursor = end;
            }
            Some("only") => {
                only = true;
                cursor += 1;
                if cursor != spans.len() {
                    return Err(err(cursor, "unexpected tokens after `only`", "unexpected"));
                }
            }
            Some("without") => {
                match spans.get(cursor + 1) {
                    Some(Spanned { tok: CodeToken::Ident(n), .. }) if n == "context" => {}
                    _ => {
                        return Err(err(cursor, "`without` requires `context`", "expected context"));
                    }
                }
                only = true;
                cursor += 2;
                if cursor != spans.len() {
                    return Err(err(cursor, "unexpected tokens after `without context`", "unexpected"));
                }
            }
            Some(_) => unreachable!(),
            None => return Err(err(cursor, "unexpected tokens after include", "unexpected")),
        }
    }

    Ok(CodeHead::Include { path_src, path_base, with_src, with_base, only })
}
