use crate::token::Span;

/// Unique identifier for AST nodes that need semantic binding.
/// Used to track which variable reference resolves to which slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

#[derive(Debug, Clone)]
pub struct Script {
    pub statements: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Line {
        parts: Vec<TextPart>,
        span: Span,
    },
    TempDecl(VarBindingData),
    SaveDecl(VarBindingData),
    Assignment(VarBindingData),
    ChoiceSet {
        choices: Vec<Choice>,
    },
}

#[derive(Debug, Clone)]
pub struct Choice {
    pub parts: Vec<TextPart>,
    pub span: Span,
    /// Nested statements to execute when this choice is selected
    pub nested: Vec<Stmt>,
}

/// A part of text content - either literal text or a variable reference
#[derive(Debug, Clone)]
pub enum TextPart {
    Literal {
        text: String,
        span: Span,
    },
    VarRef {
        id: NodeId,
        name: String,
        span: Span,
    },
}

/// A literal value in declarations
#[derive(Debug, Clone)]
pub enum Literal {
    String(String),
    Number(f64),
    Bool(bool),
}

/// Shared data for variable binding operations (declarations and assignments)
#[derive(Debug, Clone)]
pub struct VarBindingData {
    pub id: NodeId,
    pub name: String,
    pub value: Literal,
    pub span: Span,
}
