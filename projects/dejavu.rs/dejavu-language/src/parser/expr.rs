//! Expression parser over hand-written [`CodeToken`]s (Pratt-style precedence).

use dejavu_ir::NativeExpr;
use serde_json::Value;

use crate::error::ParseError;
use crate::token::CodeToken;

pub(crate) struct ExprParser<'a> {
    source: &'a str,
    file: &'a str,
    /// Absolute offset of `input` within `source`.
    base: usize,
    tokens: Vec<(CodeToken, std::ops::Range<usize>)>,
    pos: usize,
}

impl<'a> ExprParser<'a> {
    pub fn new(source: &'a str, file: &'a str, base: usize, input: &'a str) -> Result<Self, ParseError> {
        let mut tokens = Vec::new();
        let mut lex = CodeToken::lexer(input);
        while let Some(tok) = lex.next() {
            let span = lex.span();
            match tok {
                Ok(CodeToken::CodeEnd) => {
                    return Err(ParseError::at(
                        source,
                        file,
                        base + span.start,
                        span.len(),
                        "unexpected `%>` inside expression",
                        "here",
                    ));
                }
                Ok(t) => tokens.push((t, span)),
                Err(()) => {
                    return Err(ParseError::at(
                        source,
                        file,
                        base + span.start,
                        span.len().max(1),
                        "invalid token in expression",
                        "bad token",
                    ));
                }
            }
        }
        Ok(Self { source, file, base, tokens, pos: 0 })
    }

    pub fn parse(mut self) -> Result<NativeExpr, ParseError> {
        let expr = self.parse_pipe()?;
        if self.pos != self.tokens.len() {
            let span = self.peek_span();
            return Err(ParseError::at(
                self.source,
                self.file,
                span.start,
                span.len(),
                "trailing input in expression",
                "unexpected",
            ));
        }
        Ok(expr)
    }

    fn peek(&self) -> Option<&CodeToken> {
        self.tokens.get(self.pos).map(|(t, _)| t)
    }

    fn peek_span(&self) -> std::ops::Range<usize> {
        self.tokens.get(self.pos).map(|(_, s)| self.base + s.start..self.base + s.end).unwrap_or_else(|| {
            let end = self.base + self.tokens.last().map(|(_, s)| s.end).unwrap_or(0);
            end..end.saturating_add(1)
        })
    }

    fn bump(&mut self) -> Option<CodeToken> {
        let t = self.tokens.get(self.pos)?.0.clone();
        self.pos += 1;
        Some(t)
    }

    fn expect_ident(&mut self) -> Result<String, ParseError> {
        match self.bump() {
            Some(CodeToken::Ident(s)) => Ok(s),
            _ => {
                let span = self.peek_span();
                Err(ParseError::at(self.source, self.file, span.start, span.len(), "expected identifier", "expected ident"))
            }
        }
    }

    fn parse_pipe(&mut self) -> Result<NativeExpr, ParseError> {
        let mut left = self.parse_or()?;
        loop {
            if matches!(self.peek(), Some(CodeToken::PipeOp)) {
                self.bump();
                let name = self.expect_ident()?;
                let mut args = Vec::new();
                if matches!(self.peek(), Some(CodeToken::LParen)) {
                    self.bump();
                    if !matches!(self.peek(), Some(CodeToken::RParen)) {
                        loop {
                            args.push(self.parse_pipe()?);
                            if matches!(self.peek(), Some(CodeToken::Comma)) {
                                self.bump();
                                continue;
                            }
                            break;
                        }
                    }
                    if !matches!(self.bump(), Some(CodeToken::RParen)) {
                        let span = self.peek_span();
                        return Err(ParseError::at(
                            self.source,
                            self.file,
                            span.start,
                            span.len(),
                            "expected `)` after filter arguments",
                            "expected )",
                        ));
                    }
                }
                left = NativeExpr::Pipe { expression: Box::new(left), filter: name, arguments: args };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_or(&mut self) -> Result<NativeExpr, ParseError> {
        let mut left = self.parse_and()?;
        while matches!(self.peek(), Some(CodeToken::OrOr)) {
            self.bump();
            let right = self.parse_and()?;
            left = NativeExpr::Binary { operator: "||".into(), left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<NativeExpr, ParseError> {
        let mut left = self.parse_cmp()?;
        while matches!(self.peek(), Some(CodeToken::AndAnd)) {
            self.bump();
            let right = self.parse_cmp()?;
            left = NativeExpr::Binary { operator: "&&".into(), left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_cmp(&mut self) -> Result<NativeExpr, ParseError> {
        let left = self.parse_add()?;
        let op = match self.peek() {
            Some(CodeToken::EqEq) => Some("=="),
            Some(CodeToken::NotEq) => Some("!="),
            Some(CodeToken::LessEq) => Some("<="),
            Some(CodeToken::GreaterEq) => Some(">="),
            Some(CodeToken::Less) => Some("<"),
            Some(CodeToken::Greater) => Some(">"),
            Some(CodeToken::In) => Some("in"),
            _ => None,
        };
        if let Some(op) = op {
            self.bump();
            let right = self.parse_add()?;
            Ok(NativeExpr::Binary { operator: op.into(), left: Box::new(left), right: Box::new(right) })
        } else {
            Ok(left)
        }
    }

    fn parse_add(&mut self) -> Result<NativeExpr, ParseError> {
        let mut left = self.parse_mul()?;
        loop {
            match self.peek() {
                Some(CodeToken::Plus) => {
                    self.bump();
                    let right = self.parse_mul()?;
                    left = NativeExpr::Binary { operator: "+".into(), left: Box::new(left), right: Box::new(right) };
                }
                Some(CodeToken::Minus) => {
                    self.bump();
                    let right = self.parse_mul()?;
                    left = NativeExpr::Binary { operator: "-".into(), left: Box::new(left), right: Box::new(right) };
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_mul(&mut self) -> Result<NativeExpr, ParseError> {
        let mut left = self.parse_unary()?;
        loop {
            match self.peek() {
                Some(CodeToken::Star) => {
                    self.bump();
                    let right = self.parse_unary()?;
                    left = NativeExpr::Binary { operator: "*".into(), left: Box::new(left), right: Box::new(right) };
                }
                Some(CodeToken::Slash) => {
                    self.bump();
                    let right = self.parse_unary()?;
                    left = NativeExpr::Binary { operator: "/".into(), left: Box::new(left), right: Box::new(right) };
                }
                Some(CodeToken::Percent) => {
                    self.bump();
                    let right = self.parse_unary()?;
                    left = NativeExpr::Binary { operator: "%".into(), left: Box::new(left), right: Box::new(right) };
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<NativeExpr, ParseError> {
        match self.peek() {
            Some(CodeToken::Bang) => {
                self.bump();
                Ok(NativeExpr::Unary { operator: "!".into(), argument: Box::new(self.parse_unary()?) })
            }
            Some(CodeToken::Minus) => {
                self.bump();
                Ok(NativeExpr::Unary { operator: "-".into(), argument: Box::new(self.parse_unary()?) })
            }
            Some(CodeToken::Plus) => {
                self.bump();
                Ok(NativeExpr::Unary { operator: "+".into(), argument: Box::new(self.parse_unary()?) })
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Result<NativeExpr, ParseError> {
        let mut left = self.parse_primary()?;
        loop {
            match self.peek() {
                Some(CodeToken::Dot) => {
                    self.bump();
                    let property = self.expect_ident()?;
                    left = NativeExpr::Member { object: Box::new(left), property };
                }
                Some(CodeToken::LBracket) => {
                    self.bump();
                    let index = self.parse_pipe()?;
                    if !matches!(self.bump(), Some(CodeToken::RBracket)) {
                        let span = self.peek_span();
                        return Err(ParseError::at(
                            self.source,
                            self.file,
                            span.start,
                            span.len(),
                            "expected `]`",
                            "expected ]",
                        ));
                    }
                    left = NativeExpr::Index { object: Box::new(left), index: Box::new(index) };
                }
                Some(CodeToken::LParen) => {
                    self.bump();
                    let mut args = Vec::new();
                    if !matches!(self.peek(), Some(CodeToken::RParen)) {
                        loop {
                            args.push(self.parse_pipe()?);
                            if matches!(self.peek(), Some(CodeToken::Comma)) {
                                self.bump();
                                continue;
                            }
                            break;
                        }
                    }
                    if !matches!(self.bump(), Some(CodeToken::RParen)) {
                        let span = self.peek_span();
                        return Err(ParseError::at(
                            self.source,
                            self.file,
                            span.start,
                            span.len(),
                            "expected `)`",
                            "expected )",
                        ));
                    }
                    left = NativeExpr::Call { callee: Box::new(left), arguments: args };
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<NativeExpr, ParseError> {
        match self.bump() {
            Some(CodeToken::String(s)) => Ok(NativeExpr::Literal(Value::String(s))),
            Some(CodeToken::Bool(b)) => Ok(NativeExpr::Literal(Value::Bool(b))),
            Some(CodeToken::Null) => Ok(NativeExpr::Literal(Value::Null)),
            Some(CodeToken::Number(n)) => {
                let num: serde_json::Number = n.parse().map_err(|_| {
                    let span = self.peek_span();
                    ParseError::at(
                        self.source,
                        self.file,
                        span.start,
                        span.len(),
                        format!("invalid number `{n}`"),
                        "bad number",
                    )
                })?;
                Ok(NativeExpr::Literal(Value::Number(num)))
            }
            Some(CodeToken::Ident(name)) => Ok(NativeExpr::Identifier(name)),
            Some(CodeToken::LParen) => {
                let e = self.parse_pipe()?;
                if !matches!(self.bump(), Some(CodeToken::RParen)) {
                    let span = self.peek_span();
                    return Err(ParseError::at(self.source, self.file, span.start, span.len(), "expected `)`", "expected )"));
                }
                Ok(e)
            }
            _ => {
                let span = self.peek_span();
                let (start, len) = (span.start, span.len().max(1));
                Err(ParseError::at(self.source, self.file, start, len, "unexpected token in expression", "here"))
            }
        }
    }
}

pub fn parse_expr(source: &str, file: &str, base: usize, input: &str) -> Result<NativeExpr, ParseError> {
    ExprParser::new(source, file, base, input)?.parse()
}
