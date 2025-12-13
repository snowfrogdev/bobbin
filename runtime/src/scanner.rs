#[derive(Debug, Clone, Copy)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub lexeme: &'a str,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    String,
    Eof,
    NewLine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

/// Convert byte offset to (line, column), both 1-indexed.
pub fn offset_to_position(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    for (i, ch) in source.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

#[derive(Debug)]
pub enum LexicalError {
    Unexpected { message: &'static str, span: Span },
}

#[derive(Debug)]
pub struct Scanner<'a> {
    source: &'a str,
    /// Byte offset where current lexeme starts
    start: usize,
    /// Byte offset of current position
    current: usize,
    line: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn tokens(mut self) -> impl Iterator<Item = Result<Token<'a>, LexicalError>> {
        std::iter::from_fn(move || {
            let result = self.scan_token();
            match &result {
                Ok(token) if token.kind == TokenKind::Eof => None,
                _ => Some(result),
            }
        })
    }

    fn scan_token(&mut self) -> Result<Token<'a>, LexicalError> {
        self.start = self.current;

        if self.is_at_end() {
            Ok(self.make_token(TokenKind::Eof))
        } else {
            let character = self.advance();

            match character {
                Some('\n') => {
                    self.line += 1;
                    Ok(self.make_token(TokenKind::NewLine))
                }
                Some('\r') => {
                    if self.peek() == Some('\n') {
                        self.advance();
                    }
                    self.line += 1;
                    Ok(self.make_token(TokenKind::NewLine))
                }
                _ => {
                    while !self.is_at_end() && !self.is_at_newline() {
                        self.advance();
                    }
                    Ok(self.make_token(TokenKind::String))
                }
            }
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn is_at_newline(&self) -> bool {
        matches!(self.peek(), Some('\n') | Some('\r'))
    }

    fn advance(&mut self) -> Option<char> {
        let character = self.source[self.current..].chars().next()?;
        self.current += character.len_utf8();
        Some(character)
    }

    fn peek(&self) -> Option<char> {
        self.source[self.current..].chars().next()
    }

    fn make_token(&self, kind: TokenKind) -> Token<'a> {
        Token {
            kind,
            lexeme: &self.source[self.start..self.current],
            span: Span {
                start: self.start,
                end: self.current,
            },
        }
    }

    fn error(&self, message: &'static str) -> LexicalError {
        LexicalError::Unexpected {
            message,
            span: Span {
                start: self.start,
                end: self.current,
            },
        }
    }
}
