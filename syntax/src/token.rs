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
    Extern,
    If,     // if <expression>
    Elseif, // elseif <expression>
    Else,   // else

    // Logical operators (word-based)
    And, // and
    Or,  // or
    Not, // not

    // Identifiers and Literals
    Identifier,
    String,
    Number,
    True,
    False,

    // Symbols
    Equals,
    EqualEqual,   // ==
    BangEqual,    // !=
    Less,         // <
    LessEqual,    // <=
    Greater,      // >
    GreaterEqual, // >=
    Plus,         // +
    Minus,        // -
    Star,         // *
    Slash,        // /
    Percent,      // %
    OpenParen,    // (
    CloseParen,   // )
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

/// Language keywords for declarations, assignments, and control flow.
pub const KEYWORDS: &[&str] = &["save", "temp", "set", "extern", "if", "elseif", "else"];

/// Boolean literal values.
pub const BOOLEAN_LITERALS: &[&str] = &["true", "false"];
