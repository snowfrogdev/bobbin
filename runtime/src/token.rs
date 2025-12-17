#[derive(Debug, Clone, Copy)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub lexeme: &'a str,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    // Keywords
    Temp,
    Save,
    Set,

    // Identifiers and Literals
    Identifier,
    String,
    Number,
    True,
    False,

    // Symbols
    Equals,
    OpenBrace,
    CloseBrace,

    // Text (dialogue content between interpolations)
    TextSegment,

    // Structure
    Choice, // Just the "- " marker
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
