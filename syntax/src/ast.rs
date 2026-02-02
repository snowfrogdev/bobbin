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
    Line { parts: Vec<TextPart>, span: Span },
    TempDecl(VarBindingData),
    SaveDecl(VarBindingData),
    ExternDecl(ExternDeclData),
    /// Declaration of a host-provided command (callable from dialogue)
    ExternCommandDecl(ExternCommandDeclData),
    Assignment(VarBindingData),
    /// A command invocation (fire-and-forget call to host)
    CommandCall(CommandCallData),
    ChoiceSet { choices: Vec<Choice> },
    /// Conditional statement: if/elseif/else
    If {
        id: NodeId,
        /// The condition expression (must evaluate to bool)
        condition: Expr,
        /// Statements to execute if condition is true
        then_branch: Vec<Stmt>,
        /// Optional elseif branches: (condition, statements)
        elseif_branches: Vec<(Expr, Vec<Stmt>)>,
        /// Optional else branch
        else_branch: Option<Vec<Stmt>>,
        span: Span,
    },
}

#[derive(Debug, Clone)]
pub struct Choice {
    pub parts: Vec<TextPart>,
    pub span: Span,
    /// Nested statements to execute when this choice is selected
    pub nested: Vec<Stmt>,
}

/// A part of text content - literal text or expression
#[derive(Debug, Clone)]
pub enum TextPart {
    /// Plain text content
    Literal { text: String, span: Span },
    /// General expression (variable reference, comparison, etc.)
    Expr { expr: Expr, span: Span },
}

/// A literal value in declarations and expressions
#[derive(Debug, Clone)]
pub enum Literal {
    String(String),
    Number(f64),
    Bool(bool),
}

/// Binary operator for expressions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    // Logical operators (lowest precedence)
    Or,           // or
    And,          // and
    // Comparison operators
    Equal,        // ==
    NotEqual,     // !=
    Less,         // <
    LessEqual,    // <=
    Greater,      // >
    GreaterEqual, // >=
    // Arithmetic operators
    Add,          // +
    Subtract,     // -
    Multiply,     // *
    Divide,       // /
    Modulo,       // %
}

/// Unary operator for expressions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Negate, // -x
    Not,    // not x
}

/// A general expression that can be evaluated to produce a value.
///
/// This is the foundation for the expression system. Currently supports:
/// - Literals (numbers, strings, booleans)
/// - Variable references
/// - Unary operations (negation)
/// - Binary operations (comparisons, arithmetic)
#[derive(Debug, Clone)]
pub enum Expr {
    /// A literal value (number, string, boolean)
    Literal { value: Literal, span: Span },
    /// A reference to a variable
    VarRef { id: NodeId, name: String, span: Span },
    /// A unary operation (e.g., negation)
    Unary {
        id: NodeId,
        op: UnaryOp,
        expr: Box<Expr>,
        span: Span,
    },
    /// A binary operation (comparisons and arithmetic)
    Binary {
        id: NodeId,
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
        span: Span,
    },
}

impl Expr {
    /// Get the span of this expression
    pub fn span(&self) -> Span {
        match self {
            Expr::Literal { span, .. } => *span,
            Expr::VarRef { span, .. } => *span,
            Expr::Unary { span, .. } => *span,
            Expr::Binary { span, .. } => *span,
        }
    }
}


/// Shared data for variable binding operations (declarations and assignments)
#[derive(Debug, Clone)]
pub struct VarBindingData {
    pub id: NodeId,
    pub name: String,
    /// The initialization expression. Supports arithmetic, logical, and comparison
    /// operators, enabling expressions like `temp x = a + b` and `set health = health - damage`.
    pub init_expr: Expr,
    pub span: Span,
}

/// Declaration of a host-provided variable (read-only from dialogue perspective)
#[derive(Debug, Clone)]
pub struct ExternDeclData {
    pub id: NodeId,
    pub name: String,
    pub span: Span,
}

/// Declaration of a host-provided command (callable from dialogue)
///
/// Unlike `ExternDeclData` (for variables), this captures the parameter names
/// for arity validation and documentation.
#[derive(Debug, Clone)]
pub struct ExternCommandDeclData {
    pub id: NodeId,
    /// The command name (e.g., "give_gold").
    pub name: String,
    /// Parameter names for documentation (e.g., ["amount"]).
    /// The length of this vector determines the expected arity.
    pub params: Vec<String>,
    pub span: Span,
}

/// A command invocation statement.
///
/// Commands are fire-and-forget calls that trigger game-side effects.
#[derive(Debug, Clone)]
pub struct CommandCallData {
    pub id: NodeId,
    /// The command name (e.g., "give_gold").
    pub name: String,
    /// The argument expressions (evaluated at runtime).
    pub args: Vec<Expr>,
    pub span: Span,
}
