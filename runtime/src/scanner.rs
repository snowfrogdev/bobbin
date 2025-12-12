struct Token<'a> {
    kind: TokenKind,
    lexeme: &'a str,
    span: Span,
}

enum TokenKind {
    String,
    Eof,
    Error,
    NewLine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

struct Scanner<'a> {
  source: &'a str,
  /// Byte offset where current lexeme starts
  start: usize,
  /// Byte offset of current position
  current: usize,
  line: usize,
}

impl<'a> Scanner<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            start: 0,
            current: 0,
            line: 1,
        }
    }

    fn scan_token(&mut self) -> Token<'a> {
        self.start = self.current;

        if self.is_at_end() {
            self.make_token(TokenKind::Eof)
        } else {
            let character = self.advance();

            match character {
              Some('\n') => {
                self.line += 1;
                self.make_token(TokenKind::NewLine)
              }
              Some('\r') => {
                if self.peek() == Some('\n') {
                  self.advance();
                }
                self.line += 1;
                self.make_token(TokenKind::NewLine)
              }
              _ => {
                while !self.is_at_end() && !self.is_at_newline() {
                  self.advance();
                }
                self.make_token(TokenKind::String)
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

    fn error_token(&self, message: &'a str) -> Token<'a> {
        Token {
            kind: TokenKind::Error,
            lexeme: message,
            span: Span {
                start: self.start,
                end: self.current,
            },
        }
    }
}
