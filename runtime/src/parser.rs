use std::iter::Peekable;

use crate::ast::{Choice, Script, Stmt};
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
                    TokenKind::Line => {
                        statements.push(self.line_statement());
                    }
                    TokenKind::Choice => {
                        statements.push(self.choice_set());
                    }
                    TokenKind::NewLine | TokenKind::Indent | TokenKind::Dedent => {
                        // Skip newlines and indent/dedent tokens at top level
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

    fn choice_set(&mut self) -> Stmt {
        let mut choices = Vec::new();

        loop {
            let Some(Ok(token)) = self.tokens.next() else {
                unreachable!("choice_set called without verified Choice token")
            };

            if !matches!(self.tokens.peek(), Some(Ok(t)) if t.kind == TokenKind::NewLine) {
                self.errors.push(ParseError::Syntax {
                    message: "Expected newline after choice".to_string(),
                    span: token.span,
                });
                self.synchronize();
                break;
            }

            self.tokens.next(); // Consume the NewLine

            // Parse any nested content under this choice
            let nested = self.parse_nested_content();

            choices.push(Choice {
                text: token.lexeme.to_string(),
                span: token.span,
                nested,
            });

            if !matches!(self.tokens.peek(), Some(Ok(t)) if t.kind == TokenKind::Choice) {
                break;
            }
        }
        Stmt::ChoiceSet { choices }
    }

    /// Parse nested content under a choice (after Indent, before Dedent).
    /// Returns empty Vec if no nested content.
    fn parse_nested_content(&mut self) -> Vec<Stmt> {
        // Check if there's an Indent token
        if !matches!(self.tokens.peek(), Some(Ok(t)) if t.kind == TokenKind::Indent) {
            return Vec::new();
        }

        self.tokens.next(); // Consume the Indent

        let mut statements = Vec::new();

        loop {
            match self.tokens.peek() {
                None => break,
                Some(Err(_)) => {
                    if let Some(Err(e)) = self.tokens.next() {
                        self.errors.push(e.into());
                    }
                    self.synchronize();
                }
                Some(Ok(token)) => match token.kind {
                    TokenKind::Dedent => {
                        self.tokens.next(); // Consume Dedent
                        break;
                    }
                    TokenKind::Line => {
                        statements.push(self.line_statement());
                    }
                    TokenKind::Choice => {
                        statements.push(self.choice_set());
                    }
                    TokenKind::NewLine | TokenKind::Indent => {
                        self.tokens.next();
                    }
                    TokenKind::Eof => break,
                },
            }
        }

        statements
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
