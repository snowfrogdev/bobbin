use std::iter::Peekable;

use crate::ast::{Script, Stmt};
use crate::scanner::{LexicalError, Span, Token, TokenKind, offset_to_position};

#[derive(Debug)]
pub enum ParseError {
    Lexical(LexicalError),
    Syntax { message: String, span: Span },
}

impl From<LexicalError> for ParseError {
    fn from(err: LexicalError) -> Self {
        ParseError::Lexical(err)
    }
}

impl ParseError {
    pub fn format_with_source(&self, source: &str) -> String {
        match self {
            ParseError::Lexical(LexicalError::Unexpected { message, span }) => {
                let (line, col) = offset_to_position(source, span.start);
                format!("[{}:{}] lexical error: {}", line, col, message)
            }
            ParseError::Syntax { message, span } => {
                let (line, col) = offset_to_position(source, span.start);
                format!("[{}:{}] syntax error: {}", line, col, message)
            }
        }
    }
}

pub struct Parser<'a, I: Iterator<Item = Result<Token<'a>, LexicalError>>> {
    tokens: Peekable<I>,
    errors: Vec<ParseError>,
}

impl<'a, I: Iterator<Item = Result<Token<'a>, LexicalError>>> Parser<'a, I> {
    pub fn new(tokens: I) -> Self {
        Self {
            tokens: tokens.peekable(),
            errors: Vec::new(),
        }
    }

    pub fn parse(mut self) -> Result<Script, Vec<ParseError>> {
        let mut statements = Vec::new();

        loop {
            match self.tokens.peek() {
                None => break,
                Some(Err(_)) => {
                    // Consume the error
                    if let Some(Err(e)) = self.tokens.next() {
                        self.errors.push(e.into());
                    }
                    self.synchronize();
                }
                Some(Ok(token)) => match token.kind {
                    TokenKind::String => {
                        statements.push(self.line_statement());
                    }
                    TokenKind::NewLine => {
                        self.tokens.next();
                    }
                    TokenKind::Eof => break,
                },
            }
        }

        if self.errors.is_empty() {
            Ok(Script { statements })
        } else {
            Err(self.errors)
        }
    }

    fn line_statement(&mut self) -> Stmt {
        let Some(Ok(token)) = self.tokens.next() else {
            unreachable!("line_statement called without verified String token")
        };
        Stmt::Line {
            text: token.lexeme.to_string(),
            span: token.span,
        }
    }

    fn synchronize(&mut self) {
        loop {
            match self.tokens.peek() {
                None => return,
                Some(Err(_)) => {
                    if let Some(Err(e)) = self.tokens.next() {
                        self.errors.push(e.into());
                    }
                }
                Some(Ok(token)) => match token.kind {
                    TokenKind::NewLine => {
                        self.tokens.next();
                        return;
                    }
                    TokenKind::Eof => return,
                    _ => {
                        self.tokens.next();
                    }
                },
            }
        }
    }
}
