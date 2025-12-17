use crate::token::{Span, Token, TokenKind};

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

/// Scanning mode determines what tokens we expect next
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScanMode {
    /// At the physical start of a line, handle indentation
    Indentation,
    /// After indentation handled, check for keywords or text
    LineStart,
    /// After a keyword (temp/save/set), expect: identifier = literal
    Declaration,
    /// Scanning text content (dialogue lines, choice text)
    Text,
    /// Inside an interpolation {}, expect identifier
    Interpolation,
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
    /// Current scanning mode
    mode: ScanMode,
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
            mode: ScanMode::Indentation,
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
        // Handle indentation when in Indentation mode
        if self.mode == ScanMode::Indentation {
            if let Some(token) = self.handle_indentation()? {
                return Ok(token);
            }
        }

        self.start = self.current;

        if self.is_at_end() {
            return Ok(self.make_token(TokenKind::Eof));
        }

        // Handle newlines - transition to Indentation mode
        if self.consume_newline() {
            self.mode = ScanMode::Indentation;
            return Ok(self.make_token(TokenKind::NewLine));
        }

        // Dispatch based on current mode
        match self.mode {
            ScanMode::Indentation => unreachable!("should have been handled above"),
            ScanMode::LineStart => self.scan_line_start(),
            ScanMode::Declaration => self.scan_declaration_content(),
            ScanMode::Text => self.scan_text_content(),
            ScanMode::Interpolation => self.scan_interpolation_content(),
        }
    }

    /// Scan at the start of a line - check for keywords, choice marker, or text
    fn scan_line_start(&mut self) -> Result<Token<'a>, LexicalError> {
        // Check for keywords
        if self.check_keyword("temp ") {
            self.advance_n(5); // "temp "
            self.mode = ScanMode::Declaration;
            return Ok(self.make_token(TokenKind::Temp));
        }
        if self.check_keyword("save ") {
            self.advance_n(5); // "save "
            self.mode = ScanMode::Declaration;
            return Ok(self.make_token(TokenKind::Save));
        }
        if self.check_keyword("set ") {
            self.advance_n(4); // "set "
            self.mode = ScanMode::Declaration;
            return Ok(self.make_token(TokenKind::Set));
        }

        // Check for choice marker
        if self.check_keyword("- ") {
            self.advance_n(2); // "- "
            self.mode = ScanMode::Text;
            return Ok(self.make_token(TokenKind::Choice));
        }

        // Otherwise it's text content
        self.mode = ScanMode::Text;
        self.scan_text_content()
    }

    /// Scan declaration content: identifier = literal
    fn scan_declaration_content(&mut self) -> Result<Token<'a>, LexicalError> {
        self.skip_spaces();
        self.start = self.current;

        if self.is_at_end() || self.is_at_newline() {
            return Err(self.error("Unexpected end of declaration"));
        }

        let c = self.peek().unwrap();

        // Identifier
        if c.is_ascii_alphabetic() || c == '_' {
            return self.scan_identifier();
        }

        // Equals
        if c == '=' {
            self.advance();
            return Ok(self.make_token(TokenKind::Equals));
        }

        // String literal
        if c == '"' {
            return self.scan_string();
        }

        // Number literal (including negative)
        if c.is_ascii_digit() || (c == '-' && self.peek_next().is_some_and(|n| n.is_ascii_digit()))
        {
            return self.scan_number();
        }

        // Boolean literals
        if self.check_keyword("true")
            && !self
                .peek_at(4)
                .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            self.advance_n(4);
            return Ok(self.make_token(TokenKind::True));
        }
        if self.check_keyword("false")
            && !self
                .peek_at(5)
                .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            self.advance_n(5);
            return Ok(self.make_token(TokenKind::False));
        }

        Err(self.error("Unexpected character in declaration"))
    }

    /// Scan text content with interpolation support
    fn scan_text_content(&mut self) -> Result<Token<'a>, LexicalError> {
        self.start = self.current;

        if self.is_at_end() || self.is_at_newline() {
            // Empty text at end of line - switch back to line start mode
            // This shouldn't normally happen, but handle gracefully
            self.mode = ScanMode::LineStart;
            return self.scan_token();
        }

        let c = self.peek().unwrap();

        // Check for interpolation start
        if c == '{' {
            self.advance();
            // Check for escape sequence {{
            if self.peek() == Some('{') {
                self.advance();
                // Emit single { as text segment
                return Ok(Token {
                    kind: TokenKind::TextSegment,
                    lexeme: "{",
                    span: Span {
                        start: self.start,
                        end: self.current,
                    },
                });
            }
            // Start of interpolation
            self.mode = ScanMode::Interpolation;
            return Ok(self.make_token(TokenKind::OpenBrace));
        }

        // Check for }} escape sequence (standalone)
        if c == '}' {
            self.advance();
            if self.peek() == Some('}') {
                self.advance();
                // Emit single } as text segment
                return Ok(Token {
                    kind: TokenKind::TextSegment,
                    lexeme: "}",
                    span: Span {
                        start: self.start,
                        end: self.current,
                    },
                });
            }
            // Lone } is an error in text mode
            return Err(self.error("Unexpected '}' - use '}}' for literal brace"));
        }

        // Scan text segment until { or } or newline
        while !self.is_at_end() && !self.is_at_newline() {
            let c = self.peek().unwrap();
            if c == '{' || c == '}' {
                break;
            }
            self.advance();
        }

        Ok(self.make_token(TokenKind::TextSegment))
    }

    /// Scan inside an interpolation - expect identifier then }
    fn scan_interpolation_content(&mut self) -> Result<Token<'a>, LexicalError> {
        self.skip_spaces();
        self.start = self.current;

        if self.is_at_end() || self.is_at_newline() {
            return Err(self.error("Unclosed interpolation - expected '}'"));
        }

        let c = self.peek().unwrap();

        // Closing brace
        if c == '}' {
            self.advance();
            self.mode = ScanMode::Text;
            return Ok(self.make_token(TokenKind::CloseBrace));
        }

        // Identifier
        if c.is_ascii_alphabetic() || c == '_' {
            return self.scan_identifier();
        }

        Err(self.error("Expected identifier in interpolation"))
    }

    /// Scan an identifier
    fn scan_identifier(&mut self) -> Result<Token<'a>, LexicalError> {
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == '_' {
                self.advance();
            } else {
                break;
            }
        }
        Ok(self.make_token(TokenKind::Identifier))
    }

    /// Scan a string literal
    fn scan_string(&mut self) -> Result<Token<'a>, LexicalError> {
        self.advance(); // consume opening "

        while let Some(c) = self.peek() {
            if c == '"' {
                self.advance(); // consume closing "
                return Ok(self.make_token(TokenKind::String));
            }
            if c == '\\' {
                self.advance(); // consume backslash
                if !self.is_at_end() {
                    self.advance(); // consume escaped character
                }
            } else if c == '\n' || c == '\r' {
                return Err(self.error("Unterminated string - newline in string literal"));
            } else {
                self.advance();
            }
        }

        Err(self.error("Unterminated string - reached end of file"))
    }

    /// Scan a number literal (integer or float)
    fn scan_number(&mut self) -> Result<Token<'a>, LexicalError> {
        // Optional negative sign
        if self.peek() == Some('-') {
            self.advance();
        }

        // Integer part
        while self.peek().is_some_and(|c| c.is_ascii_digit()) {
            self.advance();
        }

        // Optional decimal part
        if self.peek() == Some('.') && self.peek_next().is_some_and(|c| c.is_ascii_digit()) {
            self.advance(); // consume '.'
            while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                self.advance();
            }
        }

        Ok(self.make_token(TokenKind::Number))
    }

    // =========================================================================
    // Indentation handling
    // =========================================================================

    /// Handles indentation when in Indentation mode.
    /// Returns Some(token) if an indent-related token should be emitted.
    /// Returns None to continue with normal scanning (transitions to LineStart).
    fn handle_indentation(&mut self) -> Result<Option<Token<'a>>, LexicalError> {
        // 1. Emit pending dedents first
        if self.pending_dedents > 0 {
            self.pending_dedents -= 1;
            self.start = self.current;
            return Ok(Some(self.make_token(TokenKind::Dedent)));
        }

        // 2. Process line start: skip blank lines and count leading spaces
        let spaces = match self.process_line_start()? {
            Some(count) => count,
            None => {
                // EOF reached - emit remaining dedents
                if self.indent_stack.len() > 1 {
                    self.indent_stack.pop();
                    self.pending_dedents = self.indent_stack.len() - 1;
                    self.mode = ScanMode::LineStart;
                    self.start = self.current;
                    return Ok(Some(self.make_token(TokenKind::Dedent)));
                }
                self.mode = ScanMode::LineStart;
                return Ok(None);
            }
        };

        let current_indent = self.indent_stack.last().copied().unwrap_or(0);
        self.mode = ScanMode::LineStart;
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
                // Advance past the tab and skip to end of line to avoid infinite loop
                while !self.is_at_end() && !self.is_at_newline() {
                    self.advance();
                }
                return Err(self.error("Tabs not allowed in indentation, use spaces"));
            }
            if self.is_at_end() {
                return Ok(None);
            }
            return Ok(Some(spaces));
        }
    }

    // =========================================================================
    // Helper methods
    // =========================================================================

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

    fn advance_n(&mut self, n: usize) {
        for _ in 0..n {
            self.advance();
        }
    }

    fn peek(&self) -> Option<char> {
        self.source[self.current..].chars().next()
    }

    fn peek_next(&self) -> Option<char> {
        let mut chars = self.source[self.current..].chars();
        chars.next();
        chars.next()
    }

    fn peek_at(&self, offset: usize) -> Option<char> {
        self.source[self.current..].chars().nth(offset)
    }

    /// Check if the source starting at current position matches the given string
    fn check_keyword(&self, keyword: &str) -> bool {
        self.source[self.current..].starts_with(keyword)
    }

    fn skip_spaces(&mut self) {
        while self.peek() == Some(' ') {
            self.advance();
        }
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
