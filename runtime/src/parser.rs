use std::iter::Peekable;

use crate::ast::{Choice, Literal, NodeId, Script, Stmt, TextPart};
use crate::scanner::{LexicalError, offset_to_position};
use crate::token::{Span, Token, TokenKind};

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
    next_id: usize,
}

impl<'a, I: Iterator<Item = Result<Token<'a>, LexicalError>>> Parser<'a, I> {
    pub fn new(tokens: I) -> Self {
        Self {
            tokens: tokens.peekable(),
            errors: Vec::new(),
            next_id: 0,
        }
    }

    fn next_id(&mut self) -> NodeId {
        let id = NodeId(self.next_id);
        self.next_id += 1;
        id
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
                    TokenKind::Temp => {
                        statements.push(self.temp_declaration());
                    }
                    TokenKind::TextSegment | TokenKind::OpenBrace => {
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
                    _ => {
                        // Unexpected token at statement level
                        let span = token.span;
                        self.errors.push(ParseError::Syntax {
                            message: format!("Unexpected token: {:?}", token.kind),
                            span,
                        });
                        self.tokens.next();
                    }
                },
            }
        }

        if self.errors.is_empty() {
            Ok(Script { statements })
        } else {
            Err(self.errors)
        }
    }

    /// Parse a temp declaration: temp name = value
    fn temp_declaration(&mut self) -> Stmt {
        let start_token = self.tokens.next().unwrap().unwrap(); // Consume 'temp'
        let start = start_token.span.start;
        let id = self.next_id();

        // Expect identifier
        let name = match self.tokens.peek() {
            Some(Ok(t)) if t.kind == TokenKind::Identifier => {
                let token = self.tokens.next().unwrap().unwrap();
                token.lexeme.to_string()
            }
            _ => {
                self.errors.push(ParseError::Syntax {
                    message: "Expected identifier after 'temp'".to_string(),
                    span: start_token.span,
                });
                self.synchronize();
                return Stmt::TempDecl {
                    id,
                    name: String::new(),
                    value: Literal::Bool(false),
                    span: start_token.span,
                };
            }
        };

        // Expect '='
        match self.tokens.peek() {
            Some(Ok(t)) if t.kind == TokenKind::Equals => {
                self.tokens.next();
            }
            _ => {
                self.errors.push(ParseError::Syntax {
                    message: "Expected '=' in temp declaration".to_string(),
                    span: start_token.span,
                });
                self.synchronize();
                return Stmt::TempDecl {
                    id,
                    name,
                    value: Literal::Bool(false),
                    span: start_token.span,
                };
            }
        }

        // Parse literal value
        let (value, end) = self.parse_literal();

        Stmt::TempDecl {
            id,
            name,
            value,
            span: Span { start, end },
        }
    }

    /// Parse a literal value (string, number, or boolean)
    fn parse_literal(&mut self) -> (Literal, usize) {
        match self.tokens.peek() {
            Some(Ok(t)) => match t.kind {
                TokenKind::String => {
                    let token = self.tokens.next().unwrap().unwrap();
                    // Remove quotes from the lexeme
                    let s = token.lexeme;
                    let unquoted = if s.len() >= 2 {
                        // Handle escape sequences
                        unescape_string(&s[1..s.len() - 1])
                    } else {
                        String::new()
                    };
                    (Literal::String(unquoted), token.span.end)
                }
                TokenKind::Number => {
                    let token = self.tokens.next().unwrap().unwrap();
                    let num: f64 = token.lexeme.parse().unwrap_or(0.0);
                    (Literal::Number(num), token.span.end)
                }
                TokenKind::True => {
                    let token = self.tokens.next().unwrap().unwrap();
                    (Literal::Bool(true), token.span.end)
                }
                TokenKind::False => {
                    let token = self.tokens.next().unwrap().unwrap();
                    (Literal::Bool(false), token.span.end)
                }
                _ => {
                    let span = t.span;
                    self.errors.push(ParseError::Syntax {
                        message: "Expected literal value".to_string(),
                        span,
                    });
                    (Literal::Bool(false), span.end)
                }
            },
            _ => {
                self.errors.push(ParseError::Syntax {
                    message: "Expected literal value".to_string(),
                    span: Span { start: 0, end: 0 },
                });
                (Literal::Bool(false), 0)
            }
        }
    }

    /// Parse a line statement (text content with possible interpolation)
    fn line_statement(&mut self) -> Stmt {
        let (parts, span) = self.parse_text_parts();
        Stmt::Line { parts, span }
    }

    /// Parse text parts until newline (TextSegment, interpolations)
    fn parse_text_parts(&mut self) -> (Vec<TextPart>, Span) {
        let mut parts = Vec::new();
        let mut start: Option<usize> = None;
        let mut end: usize = 0;

        loop {
            match self.tokens.peek() {
                Some(Ok(t)) => match t.kind {
                    TokenKind::TextSegment => {
                        let token = self.tokens.next().unwrap().unwrap();
                        if start.is_none() {
                            start = Some(token.span.start);
                        }
                        end = token.span.end;
                        parts.push(TextPart::Literal {
                            text: token.lexeme.to_string(),
                            span: token.span,
                        });
                    }
                    TokenKind::OpenBrace => {
                        let open = self.tokens.next().unwrap().unwrap();
                        if start.is_none() {
                            start = Some(open.span.start);
                        }

                        // Expect identifier
                        match self.tokens.peek() {
                            Some(Ok(t)) if t.kind == TokenKind::Identifier => {
                                let id_token = self.tokens.next().unwrap().unwrap();
                                let var_name = id_token.lexeme.to_string();

                                // Expect close brace
                                match self.tokens.peek() {
                                    Some(Ok(t)) if t.kind == TokenKind::CloseBrace => {
                                        let close = self.tokens.next().unwrap().unwrap();
                                        end = close.span.end;
                                        parts.push(TextPart::VarRef {
                                            id: self.next_id(),
                                            name: var_name,
                                            span: Span {
                                                start: open.span.start,
                                                end: close.span.end,
                                            },
                                        });
                                    }
                                    _ => {
                                        self.errors.push(ParseError::Syntax {
                                            message: "Expected '}' after variable name".to_string(),
                                            span: id_token.span,
                                        });
                                        end = id_token.span.end;
                                    }
                                }
                            }
                            _ => {
                                self.errors.push(ParseError::Syntax {
                                    message: "Expected variable name after '{'".to_string(),
                                    span: open.span,
                                });
                                end = open.span.end;
                            }
                        }
                    }
                    TokenKind::NewLine | TokenKind::Eof | TokenKind::Dedent => {
                        // End of text content
                        break;
                    }
                    _ => {
                        // Unexpected token in text
                        break;
                    }
                },
                Some(Err(_)) => {
                    if let Some(Err(e)) = self.tokens.next() {
                        self.errors.push(e.into());
                    }
                    break;
                }
                None => break,
            }
        }

        let span = Span {
            start: start.unwrap_or(0),
            end,
        };
        (parts, span)
    }

    fn choice_set(&mut self) -> Stmt {
        let mut choices = Vec::new();

        loop {
            // Consume the Choice token ("- ")
            let choice_token = self.tokens.next().unwrap().unwrap();
            let start = choice_token.span.start;

            // Parse the choice text (may contain interpolation)
            let (parts, text_span) = self.parse_text_parts();
            let end = if text_span.end > 0 {
                text_span.end
            } else {
                choice_token.span.end
            };

            // Expect newline after choice text
            if !matches!(self.tokens.peek(), Some(Ok(t)) if t.kind == TokenKind::NewLine) {
                self.errors.push(ParseError::Syntax {
                    message: "Expected newline after choice".to_string(),
                    span: Span { start, end },
                });
                self.synchronize();
                break;
            }

            self.tokens.next(); // Consume the NewLine

            // Parse any nested content under this choice
            let nested = self.parse_nested_content();

            choices.push(Choice {
                parts,
                span: Span { start, end },
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
                    TokenKind::Temp => {
                        statements.push(self.temp_declaration());
                    }
                    TokenKind::TextSegment | TokenKind::OpenBrace => {
                        statements.push(self.line_statement());
                    }
                    TokenKind::Choice => {
                        statements.push(self.choice_set());
                    }
                    TokenKind::NewLine | TokenKind::Indent => {
                        self.tokens.next();
                    }
                    TokenKind::Eof => break,
                    _ => {
                        // Skip unexpected tokens
                        self.tokens.next();
                    }
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

/// Unescape a string literal (handle \n, \t, \", \\)
fn unescape_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('t') => result.push('\t'),
                Some('r') => result.push('\r'),
                Some('"') => result.push('"'),
                Some('\\') => result.push('\\'),
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }

    result
}
