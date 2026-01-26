use crate::diagnostic::{Diagnostic, DiagnosticContext, IntoDiagnostic};
use crate::token::{Span, Token, TokenKind};

#[derive(Debug, Clone)]
pub enum LexicalError {
    Unexpected { message: &'static str, span: Span },
}

impl IntoDiagnostic for LexicalError {
    fn into_diagnostic(self, _ctx: &DiagnosticContext) -> Diagnostic {
        match self {
            LexicalError::Unexpected { message, span } => {
                let mut diag =
                    Diagnostic::error(format!("lexical error: {}", message), span, message);

                // Add helpful notes for specific error types
                if message.contains("Tabs not allowed") {
                    diag = diag.with_note("Bobbin uses spaces for indentation, not tabs");
                } else if message.contains("Unterminated string") {
                    diag = diag.with_note("Strings cannot span multiple lines");
                } else if message.contains("Unexpected '}'") {
                    diag = diag.with_suggestion("use '}}' for a literal brace in text", span, "}}");
                }

                diag
            }
        }
    }
}

/// Scanning mode determines what tokens we expect next
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScanMode {
    /// At the physical start of a line, handle indentation
    Indentation,
    /// After indentation handled, check for keywords or text
    LineStart,
    /// After a keyword (temp/save/set), expect: identifier =
    /// After `=`, transitions to Condition mode to scan the expression.
    Declaration,
    /// After extern keyword, expect: identifier only (no initializer)
    ExternDeclaration,
    /// Scanning text content (dialogue lines, choice text)
    Text,
    /// Inside an interpolation {}, expect identifier
    Interpolation,
    /// After if/elseif keyword, scan condition expression
    Condition,
}

#[derive(Debug)]
pub struct Scanner<'a> {
    source: &'a str,
    /// Byte offset where current lexeme starts
    start: usize,
    /// Byte offset of current position
    current: usize,
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
            ScanMode::ExternDeclaration => self.scan_extern_declaration(),
            ScanMode::Text => self.scan_text_content(),
            ScanMode::Interpolation => self.scan_interpolation_content(),
            ScanMode::Condition => self.scan_condition(),
        }
    }

    /// Scan at the start of a line - check for keywords, choice marker, or text
    fn scan_line_start(&mut self) -> Result<Token<'a>, LexicalError> {
        // Declaration keywords
        if let Some(tok) = self.try_keyword("temp", TokenKind::Temp, ScanMode::Declaration) {
            return Ok(tok);
        }
        if let Some(tok) = self.try_keyword("save", TokenKind::Save, ScanMode::Declaration) {
            return Ok(tok);
        }
        if let Some(tok) = self.try_keyword("set", TokenKind::Set, ScanMode::Declaration) {
            return Ok(tok);
        }
        if let Some(tok) =
            self.try_keyword("extern", TokenKind::Extern, ScanMode::ExternDeclaration)
        {
            return Ok(tok);
        }

        // Conditional keywords - transition to Condition mode to scan expression
        if let Some(tok) = self.try_keyword("if", TokenKind::If, ScanMode::Condition) {
            return Ok(tok);
        }
        if let Some(tok) = self.try_keyword("elseif", TokenKind::Elseif, ScanMode::Condition) {
            return Ok(tok);
        }
        // "else" has no expression - just emit token and stay at LineStart for next line
        if let Some(tok) = self.try_else_keyword() {
            return Ok(tok);
        }

        // Choice marker
        if let Some(tok) = self.try_keyword("-", TokenKind::Choice, ScanMode::Text) {
            return Ok(tok);
        }

        // Otherwise it's text content
        self.mode = ScanMode::Text;
        self.scan_text_content()
    }

    /// Try to match "else" keyword (no expression follows).
    /// "else" can be followed by newline, or be at end of line.
    fn try_else_keyword(&mut self) -> Option<Token<'a>> {
        let remaining = &self.source[self.current..];
        if !remaining.starts_with("else") {
            return None;
        }
        // Must be at end of content or followed by whitespace/newline
        let after = &remaining[4..];
        if after.is_empty()
            || after.starts_with('\n')
            || after.starts_with('\r')
            || after.starts_with(' ')
        {
            self.advance_n(4);
            let token = self.make_token(TokenKind::Else);
            // Skip any trailing spaces (though there shouldn't be content after else)
            self.skip_spaces();
            self.mode = ScanMode::LineStart;
            return Some(token);
        }
        None
    }

    /// Try to match a keyword followed by space. Returns token if matched.
    /// The token lexeme contains only the keyword (not the trailing space).
    fn try_keyword(
        &mut self,
        keyword: &str,
        kind: TokenKind,
        next_mode: ScanMode,
    ) -> Option<Token<'a>> {
        let remaining = &self.source[self.current..];

        // Must match keyword
        if !remaining.starts_with(keyword) {
            return None;
        }

        // Must be followed by space (word boundary)
        if !remaining[keyword.len()..].starts_with(' ') {
            return None;
        }

        self.advance_n(keyword.len());
        let token = self.make_token(kind);
        self.skip_spaces();
        self.mode = next_mode;
        Some(token)
    }

    /// Scan declaration content: identifier = expression
    /// After the `=` sign, transitions to Condition mode to scan the expression.
    fn scan_declaration_content(&mut self) -> Result<Token<'a>, LexicalError> {
        self.skip_spaces();
        self.start = self.current;

        if self.is_at_end() || self.is_at_newline() {
            return Err(self.error("Unexpected end of declaration"));
        }

        let c = self.peek().unwrap();

        // Equals - after this, transition to Condition mode for expression scanning
        if c == '=' {
            self.advance();
            // Switch to Condition mode which handles all expression tokens
            // and transitions back to LineStart on newline
            self.mode = ScanMode::Condition;
            return Ok(self.make_token(TokenKind::Equals));
        }

        // Identifier (variable name before the =)
        if c.is_ascii_alphabetic() || c == '_' {
            return self.scan_identifier();
        }

        // Error recovery: advance past the invalid character to avoid infinite loop
        self.advance();
        Err(self.error("Unexpected character in declaration"))
    }

    /// Scan extern declaration content: identifier only (no initializer)
    fn scan_extern_declaration(&mut self) -> Result<Token<'a>, LexicalError> {
        self.skip_spaces();
        self.start = self.current;

        if self.is_at_end() || self.is_at_newline() {
            return Err(self.error("Expected identifier after 'extern'"));
        }

        let c = self.peek().unwrap();
        if c.is_ascii_alphabetic() || c == '_' {
            self.mode = ScanMode::LineStart;
            return self.scan_identifier();
        }

        // Error recovery: advance past the invalid character to avoid infinite loop
        self.advance();
        Err(self.error("Expected identifier after 'extern'"))
    }

    /// Scan condition expression after if/elseif keyword.
    /// Reuses interpolation scanning for expression tokens.
    fn scan_condition(&mut self) -> Result<Token<'a>, LexicalError> {
        self.skip_spaces();
        self.start = self.current;

        if self.is_at_end() || self.is_at_newline() {
            // End of condition expression - transition back to line start
            self.mode = ScanMode::LineStart;
            return self.scan_token();
        }

        let current_char = self.peek().unwrap();

        // Operators (reuse from interpolation)
        // == operator
        if current_char == '=' && self.peek_next() == Some('=') {
            self.advance();
            self.advance();
            return Ok(self.make_token(TokenKind::EqualEqual));
        }

        // != operator
        if current_char == '!' && self.peek_next() == Some('=') {
            self.advance();
            self.advance();
            return Ok(self.make_token(TokenKind::BangEqual));
        }

        // <= operator
        if current_char == '<' && self.peek_next() == Some('=') {
            self.advance();
            self.advance();
            return Ok(self.make_token(TokenKind::LessEqual));
        }

        // < operator
        if current_char == '<' {
            self.advance();
            return Ok(self.make_token(TokenKind::Less));
        }

        // >= operator
        if current_char == '>' && self.peek_next() == Some('=') {
            self.advance();
            self.advance();
            return Ok(self.make_token(TokenKind::GreaterEqual));
        }

        // > operator
        if current_char == '>' {
            self.advance();
            return Ok(self.make_token(TokenKind::Greater));
        }

        // Arithmetic operators
        if current_char == '+' {
            self.advance();
            return Ok(self.make_token(TokenKind::Plus));
        }

        if current_char == '-' {
            self.advance();
            return Ok(self.make_token(TokenKind::Minus));
        }

        if current_char == '*' {
            self.advance();
            return Ok(self.make_token(TokenKind::Star));
        }

        if current_char == '/' {
            self.advance();
            return Ok(self.make_token(TokenKind::Slash));
        }

        if current_char == '%' {
            self.advance();
            return Ok(self.make_token(TokenKind::Percent));
        }

        // Parentheses for grouping
        if current_char == '(' {
            self.advance();
            return Ok(self.make_token(TokenKind::OpenParen));
        }

        if current_char == ')' {
            self.advance();
            return Ok(self.make_token(TokenKind::CloseParen));
        }

        // Reject lone = with helpful error
        if current_char == '=' {
            self.advance();
            return Err(self.error("Assignment not allowed in condition - did you mean '=='?"));
        }

        // Reject lone ! with helpful error
        if current_char == '!' {
            self.advance();
            return Err(self.error("Expected '!=' for inequality"));
        }

        // String literal
        if current_char == '"' {
            return self.scan_string();
        }

        // Number literal
        if current_char.is_ascii_digit() {
            return self.scan_number();
        }

        // Identifier or keyword (true/false/and/or/not)
        if current_char.is_ascii_alphabetic() || current_char == '_' {
            return self.scan_identifier_or_keyword();
        }

        // Error recovery
        self.advance();
        Err(self.error("Invalid character in condition"))
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

    /// Scan inside an interpolation - handles operators, literals, and identifiers
    fn scan_interpolation_content(&mut self) -> Result<Token<'a>, LexicalError> {
        self.skip_spaces();
        self.start = self.current;

        if self.is_at_end() || self.is_at_newline() {
            self.mode = ScanMode::Text;
            return Err(self.error("Unclosed interpolation - expected '}'"));
        }

        let current_char = self.peek().unwrap();

        // Closing brace - end interpolation
        if current_char == '}' {
            self.advance();
            self.mode = ScanMode::Text;
            return Ok(self.make_token(TokenKind::CloseBrace));
        }

        // == operator (must check before lone =)
        if current_char == '=' && self.peek_next() == Some('=') {
            self.advance();
            self.advance();
            return Ok(self.make_token(TokenKind::EqualEqual));
        }

        // != operator
        if current_char == '!' && self.peek_next() == Some('=') {
            self.advance();
            self.advance();
            return Ok(self.make_token(TokenKind::BangEqual));
        }

        // <= operator (check before lone <)
        if current_char == '<' && self.peek_next() == Some('=') {
            self.advance();
            self.advance();
            return Ok(self.make_token(TokenKind::LessEqual));
        }

        // < operator
        if current_char == '<' {
            self.advance();
            return Ok(self.make_token(TokenKind::Less));
        }

        // >= operator (check before lone >)
        if current_char == '>' && self.peek_next() == Some('=') {
            self.advance();
            self.advance();
            return Ok(self.make_token(TokenKind::GreaterEqual));
        }

        // > operator
        if current_char == '>' {
            self.advance();
            return Ok(self.make_token(TokenKind::Greater));
        }

        // Arithmetic operators
        if current_char == '+' {
            self.advance();
            return Ok(self.make_token(TokenKind::Plus));
        }

        // Minus operator - always emit as token, parser handles unary vs binary
        if current_char == '-' {
            self.advance();
            return Ok(self.make_token(TokenKind::Minus));
        }

        if current_char == '*' {
            self.advance();
            return Ok(self.make_token(TokenKind::Star));
        }

        if current_char == '/' {
            self.advance();
            return Ok(self.make_token(TokenKind::Slash));
        }

        if current_char == '%' {
            self.advance();
            return Ok(self.make_token(TokenKind::Percent));
        }

        // Parentheses for grouping
        if current_char == '(' {
            self.advance();
            return Ok(self.make_token(TokenKind::OpenParen));
        }

        if current_char == ')' {
            self.advance();
            return Ok(self.make_token(TokenKind::CloseParen));
        }

        // Reject lone = with helpful error
        if current_char == '=' {
            self.advance();
            return Err(self.error("Assignment not allowed in interpolation - did you mean '=='?"));
        }

        // Reject lone ! with helpful error
        if current_char == '!' {
            self.advance();
            return Err(self.error("Expected '!=' for inequality"));
        }

        // String literal
        if current_char == '"' {
            return self.scan_string();
        }

        // Number literal (positive only - minus is handled as operator above)
        if current_char.is_ascii_digit() {
            return self.scan_number();
        }

        // Identifier or keyword (true/false)
        if current_char.is_ascii_alphabetic() || current_char == '_' {
            return self.scan_identifier_or_keyword();
        }

        // Error recovery
        self.advance();
        Err(self.error("Invalid character in interpolation"))
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

    /// Scan an identifier, then check if it's a keyword (true/false).
    /// Used in declaration context where both identifiers and boolean literals are valid.
    fn scan_identifier_or_keyword(&mut self) -> Result<Token<'a>, LexicalError> {
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == '_' {
                self.advance();
            } else {
                break;
            }
        }

        let lexeme = &self.source[self.start..self.current];
        let kind = match lexeme {
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "and" => TokenKind::And,
            "or" => TokenKind::Or,
            "not" => TokenKind::Not,
            _ => TokenKind::Identifier,
        };

        Ok(self.make_token(kind))
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
            // Transition to LineStart when all dedents are emitted
            if self.pending_dedents == 0 {
                self.mode = ScanMode::LineStart;
            }
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
        self.start = self.current;

        if spaces > current_indent {
            // Indent: push new level
            self.indent_stack.push(spaces);
            self.mode = ScanMode::LineStart;
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
            // Only transition to LineStart when all dedents are emitted
            if self.pending_dedents == 0 {
                self.mode = ScanMode::LineStart;
            }
            Ok(Some(self.make_token(TokenKind::Dedent)))
        } else {
            // Same level - no token
            self.mode = ScanMode::LineStart;
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
                true
            }
            Some('\r') => {
                self.advance();
                if self.peek() == Some('\n') {
                    self.advance();
                }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn scan_tokens(source: &str) -> Vec<Token<'_>> {
        Scanner::new(source)
            .tokens()
            .filter_map(|r| r.ok())
            .collect()
    }

    fn token_kinds(source: &str) -> Vec<TokenKind> {
        scan_tokens(source).into_iter().map(|t| t.kind).collect()
    }

    // === Equality Operator Tests ===

    #[test]
    fn equality_operator_in_interpolation() {
        let kinds = token_kinds("{x == y}");
        assert!(kinds.contains(&TokenKind::EqualEqual));
    }

    #[test]
    fn inequality_operator_in_interpolation() {
        let kinds = token_kinds("{x != y}");
        assert!(kinds.contains(&TokenKind::BangEqual));
    }

    #[test]
    fn equality_with_string_literal() {
        let kinds = token_kinds("{x == \"hello\"}");
        assert!(kinds.contains(&TokenKind::EqualEqual));
        assert!(kinds.contains(&TokenKind::String));
    }

    #[test]
    fn equality_with_number_literal() {
        let kinds = token_kinds("{x == 42}");
        assert!(kinds.contains(&TokenKind::EqualEqual));
        assert!(kinds.contains(&TokenKind::Number));
    }

    #[test]
    fn equality_with_negative_number() {
        let kinds = token_kinds("{x == -5}");
        assert!(kinds.contains(&TokenKind::EqualEqual));
        assert!(kinds.contains(&TokenKind::Number));
    }

    #[test]
    fn equality_with_boolean_true() {
        let kinds = token_kinds("{x == true}");
        assert!(kinds.contains(&TokenKind::EqualEqual));
        assert!(kinds.contains(&TokenKind::True));
    }

    #[test]
    fn equality_with_boolean_false() {
        let kinds = token_kinds("{x == false}");
        assert!(kinds.contains(&TokenKind::EqualEqual));
        assert!(kinds.contains(&TokenKind::False));
    }

    // === Error Case Tests ===

    #[test]
    fn lone_equals_produces_error() {
        let results: Vec<_> = Scanner::new("{x = y}").tokens().collect();
        let has_error = results.iter().any(|r| r.is_err());
        assert!(has_error, "Expected error for lone = in interpolation");
    }

    #[test]
    fn lone_bang_produces_error() {
        let results: Vec<_> = Scanner::new("{x ! y}").tokens().collect();
        let has_error = results.iter().any(|r| r.is_err());
        assert!(has_error, "Expected error for lone ! in interpolation");
    }

    // === Edge Case Tests ===

    #[test]
    fn no_spaces_around_operator() {
        let kinds = token_kinds("{x==y}");
        assert!(kinds.contains(&TokenKind::EqualEqual));
    }

    #[test]
    fn multiple_spaces_around_operator() {
        let kinds = token_kinds("{x  ==  y}");
        assert!(kinds.contains(&TokenKind::EqualEqual));
    }

    // === Comparison Operator Tests ===

    #[test]
    fn less_than_operator_in_interpolation() {
        let kinds = token_kinds("{x < y}");
        assert!(kinds.contains(&TokenKind::Less));
    }

    #[test]
    fn less_equal_operator_in_interpolation() {
        let kinds = token_kinds("{x <= y}");
        assert!(kinds.contains(&TokenKind::LessEqual));
    }

    #[test]
    fn greater_than_operator_in_interpolation() {
        let kinds = token_kinds("{x > y}");
        assert!(kinds.contains(&TokenKind::Greater));
    }

    #[test]
    fn greater_equal_operator_in_interpolation() {
        let kinds = token_kinds("{x >= y}");
        assert!(kinds.contains(&TokenKind::GreaterEqual));
    }
}
