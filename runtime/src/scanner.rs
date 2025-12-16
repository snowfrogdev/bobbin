#[derive(Debug, Clone, Copy)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub lexeme: &'a str,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Line,
    Choice,
    Indent,
    Dedent,
    NewLine,
    Eof,
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
    indent_stack: Vec<usize>,
    pending_dedents: usize,
    at_line_start: bool,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            start: 0,
            current: 0,
            line: 1,
            indent_stack: vec![0],
            pending_dedents: 0,
            at_line_start: true,
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
        // Handle all indentation concerns first
        if let Some(token) = self.handle_indentation()? {
            return Ok(token);
        }

        self.start = self.current;

        if self.is_at_end() {
            return Ok(self.make_token(TokenKind::Eof));
        }

        if self.consume_newline() {
            self.at_line_start = true;
            return Ok(self.make_token(TokenKind::NewLine));
        }

        match self.advance() {
            Some('-') => {
                if self.peek() == Some(' ') {
                    self.advance();
                    self.start = self.current;
                    while !self.is_at_end() && !self.is_at_newline() {
                        self.advance();
                    }
                    Ok(self.make_token(TokenKind::Choice))
                } else {
                    Err(self.error("Choice marker '-' must be followed by a space"))
                }
            }
            _ => {
                while !self.is_at_end() && !self.is_at_newline() {
                    self.advance();
                }
                Ok(self.make_token(TokenKind::Line))
            }
        }
    }

    /// Handles all indentation-related concerns at the start of scan_token.
    /// Returns Some(token) if an indent-related token should be emitted.
    /// Returns None to continue with normal scanning.
    fn handle_indentation(&mut self) -> Result<Option<Token<'a>>, LexicalError> {
        // 1. Emit pending dedents first
        if self.pending_dedents > 0 {
            self.pending_dedents -= 1;
            self.start = self.current;
            return Ok(Some(self.make_token(TokenKind::Dedent)));
        }

        // 2. Not at line start? Nothing to do
        if !self.at_line_start {
            return Ok(None);
        }

        // 3. Process line start: skip blank lines and count leading spaces
        let spaces = match self.process_line_start()? {
            Some(count) => count,
            None => {
                // EOF reached - emit remaining dedents
                if self.indent_stack.len() > 1 {
                    self.indent_stack.pop();
                    self.pending_dedents = self.indent_stack.len() - 1;
                    self.at_line_start = false;
                    self.start = self.current;
                    return Ok(Some(self.make_token(TokenKind::Dedent)));
                }
                return Ok(None);
            }
        };

        let current_indent = self.indent_stack.last().copied().unwrap_or(0);
        self.at_line_start = false;
        self.start = self.current;

        if spaces > current_indent {
            // Indent: push new level
            self.indent_stack.push(spaces);
            Ok(Some(self.make_token(TokenKind::Indent)))
        } else if spaces < current_indent {
            // Dedent: pop until we find matching level
            while self
                .indent_stack
                .last()
                .is_some_and(|&level| level > spaces)
            {
                self.indent_stack.pop();
                self.pending_dedents += 1;
            }
            if self.indent_stack.last().copied() != Some(spaces) {
                return Err(self.error("Inconsistent indentation"));
            }
            self.pending_dedents -= 1; // We emit one now
            Ok(Some(self.make_token(TokenKind::Dedent)))
        } else {
            // Same level - no token
            Ok(None)
        }
    }

    /// Skips blank lines and returns the leading space count of the first content line.
    /// Returns None if EOF is reached.
    fn process_line_start(&mut self) -> Result<Option<usize>, LexicalError> {
        loop {
            self.start = self.current;
            let mut spaces = 0;
            while self.peek() == Some(' ') {
                self.advance();
                spaces += 1;
            }
            if self.consume_newline() {
                continue;
            }
            if self.peek() == Some('\t') {
                return Err(self.error("Tabs not allowed in indentation, use spaces"));
            }
            if self.is_at_end() {
                return Ok(None);
            }
            return Ok(Some(spaces));
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn is_at_newline(&self) -> bool {
        matches!(self.peek(), Some('\n') | Some('\r'))
    }

    /// Consumes a newline (\n or \r\n) if present. Returns true if consumed.
    fn consume_newline(&mut self) -> bool {
        match self.peek() {
            Some('\n') => {
                self.advance();
                self.line += 1;
                true
            }
            Some('\r') => {
                self.advance();
                if self.peek() == Some('\n') {
                    self.advance();
                }
                self.line += 1;
                true
            }
            _ => false,
        }
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
