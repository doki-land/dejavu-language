//! Hand-written tokens + lexer for code inside `<% ... %>`.

use std::ops::Range;

/// Lexical token inside a code delimiter.
#[derive(Debug, PartialEq, Clone)]
pub enum CodeToken {
    CodeEnd,
    PipeOp,
    OrOr,
    AndAnd,
    EqEq,
    NotEq,
    LessEq,
    GreaterEq,
    Less,
    Greater,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Bang,
    Dot,
    Comma,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Bool(bool),
    Null,
    In,
    Ident(String),
    Number(String),
    String(String),
}

impl CodeToken {
    /// Lex `input` (API kept compatible with the former logos entry point).
    pub fn lexer(input: &str) -> CodeLexer<'_> {
        CodeLexer::new(input)
    }
}

/// Byte-oriented scanner; yields `Ok(token)` or `Err(())` for invalid input.
pub struct CodeLexer<'a> {
    input: &'a str,
    bytes: &'a [u8],
    pos: usize,
    /// Span of the last token produced by [`Self::next`].
    last_span: Range<usize>,
}

impl<'a> CodeLexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, bytes: input.as_bytes(), pos: 0, last_span: 0..0 }
    }

    pub fn span(&self) -> Range<usize> {
        self.last_span.clone()
    }

    pub fn next(&mut self) -> Option<Result<CodeToken, ()>> {
        self.skip_ws();
        if self.pos >= self.bytes.len() {
            return None;
        }
        let start = self.pos;
        let tok = match self.scan_one() {
            Ok(t) => {
                self.last_span = start..self.pos;
                Ok(t)
            }
            Err(()) => {
                // Advance at least one byte so callers can recover / report.
                if self.pos == start {
                    self.pos = (start + 1).min(self.bytes.len());
                }
                self.last_span = start..self.pos;
                Err(())
            }
        };
        Some(tok)
    }

    fn skip_ws(&mut self) {
        while self.pos < self.bytes.len() {
            match self.bytes[self.pos] {
                b' ' | b'\t' | b'\r' | b'\n' => self.pos += 1,
                _ => break,
            }
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn peek2(&self) -> Option<(u8, u8)> {
        if self.pos + 1 < self.bytes.len() { Some((self.bytes[self.pos], self.bytes[self.pos + 1])) } else { None }
    }

    fn bump(&mut self) -> Option<u8> {
        let b = self.peek()?;
        self.pos += 1;
        Some(b)
    }

    fn scan_one(&mut self) -> Result<CodeToken, ()> {
        // Two-byte operators / closers first.
        if let Some((a, b)) = self.peek2() {
            let two = match (a, b) {
                (b'%', b'>') => Some(CodeToken::CodeEnd),
                (b'|', b'>') => Some(CodeToken::PipeOp),
                (b'|', b'|') => Some(CodeToken::OrOr),
                (b'&', b'&') => Some(CodeToken::AndAnd),
                (b'=', b'=') => Some(CodeToken::EqEq),
                (b'!', b'=') => Some(CodeToken::NotEq),
                (b'<', b'=') => Some(CodeToken::LessEq),
                (b'>', b'=') => Some(CodeToken::GreaterEq),
                _ => None,
            };
            if let Some(tok) = two {
                self.pos += 2;
                return Ok(tok);
            }
        }

        let Some(b) = self.bump() else {
            return Err(());
        };

        Ok(match b {
            b'<' => CodeToken::Less,
            b'>' => CodeToken::Greater,
            b'+' => CodeToken::Plus,
            b'-' => CodeToken::Minus,
            b'*' => CodeToken::Star,
            b'/' => CodeToken::Slash,
            b'%' => CodeToken::Percent,
            b'!' => CodeToken::Bang,
            b'.' => CodeToken::Dot,
            b',' => CodeToken::Comma,
            b'(' => CodeToken::LParen,
            b')' => CodeToken::RParen,
            b'[' => CodeToken::LBracket,
            b']' => CodeToken::RBracket,
            b'"' | b'\'' => self.scan_string(b)?,
            c if c.is_ascii_digit() => self.scan_number(c),
            c if is_ident_start(c) => self.scan_ident_or_keyword(c),
            _ => return Err(()),
        })
    }

    fn scan_ident_or_keyword(&mut self, _first: u8) -> CodeToken {
        let start = self.pos - 1;
        while matches!(self.peek(), Some(c) if is_ident_continue(c)) {
            self.pos += 1;
        }
        let s = &self.input[start..self.pos];
        match s {
            "true" => CodeToken::Bool(true),
            "false" => CodeToken::Bool(false),
            "null" => CodeToken::Null,
            "in" => CodeToken::In,
            _ => CodeToken::Ident(s.to_string()),
        }
    }

    fn scan_number(&mut self, _first: u8) -> CodeToken {
        let start = self.pos - 1;
        while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
            self.pos += 1;
        }
        if self.peek() == Some(b'.') {
            let after_dot = self.bytes.get(self.pos + 1).copied();
            if matches!(after_dot, Some(c) if c.is_ascii_digit()) {
                self.pos += 1; // '.'
                while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
                    self.pos += 1;
                }
            }
        }
        CodeToken::Number(self.input[start..self.pos].to_string())
    }

    fn scan_string(&mut self, quote: u8) -> Result<CodeToken, ()> {
        let start = self.pos; // content start (after opening quote)
        loop {
            match self.bump() {
                None => return Err(()), // unclosed
                Some(c) if c == quote => {
                    let inner = &self.input[start..self.pos - 1];
                    return Ok(CodeToken::String(inner.to_string()));
                }
                Some(b'\\') => {
                    if self.bump().is_none() {
                        return Err(());
                    }
                }
                Some(_) => {}
            }
        }
    }
}

fn is_ident_start(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_'
}

fn is_ident_continue(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex_all(input: &str) -> Vec<CodeToken> {
        let mut lex = CodeToken::lexer(input);
        let mut out = Vec::new();
        while let Some(tok) = lex.next() {
            out.push(tok.expect("token"));
        }
        out
    }

    #[test]
    fn operators_and_keywords() {
        assert_eq!(
            lex_all("true || false && x |> y"),
            vec![
                CodeToken::Bool(true),
                CodeToken::OrOr,
                CodeToken::Bool(false),
                CodeToken::AndAnd,
                CodeToken::Ident("x".into()),
                CodeToken::PipeOp,
                CodeToken::Ident("y".into()),
            ]
        );
    }

    #[test]
    fn strings_and_numbers() {
        assert_eq!(
            lex_all(r#"1.5 "a\"b" 'c'"#),
            vec![CodeToken::Number("1.5".into()), CodeToken::String(r#"a\"b"#.into()), CodeToken::String("c".into()),]
        );
    }

    #[test]
    fn code_end() {
        assert_eq!(lex_all("%>"), vec![CodeToken::CodeEnd]);
    }
}
