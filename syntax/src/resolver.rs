use std::collections::HashMap;

use crate::ast::{BinaryOp, Choice, ExternDeclData, Literal, NodeId, Script, Stmt, TextPart, UnaryOp, VarBindingData};
use crate::diagnostic::{Diagnostic, DiagnosticContext, IntoDiagnostic};
use crate::token::Span;

/// The type of a variable's value, inferred from its initializer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    Number,
    String,
    Bool,
}

impl ValueType {
    /// Extract the type from a literal value.
    pub fn from_literal(lit: &Literal) -> Self {
        match lit {
            Literal::Number(_) => ValueType::Number,
            Literal::String(_) => ValueType::String,
            Literal::Bool(_) => ValueType::Bool,
        }
    }

    /// Human-readable type name for error messages.
    pub fn name(&self) -> &'static str {
        match self {
            ValueType::Number => "number",
            ValueType::String => "string",
            ValueType::Bool => "bool",
        }
    }
}

#[derive(Debug, Clone)]
pub enum SemanticError {
    UndefinedVariable {
        name: String,
        span: Span,
    },
    Shadowing {
        name: String,
        span: Span,
        original: Span,
    },
    AssignmentToExtern {
        name: String,
        span: Span,
    },
    TypeMismatch {
        left_name: String,
        left_type: ValueType,
        right_desc: String,
        right_type: ValueType,
        span: Span,
    },
    /// Ordering comparison operators (<, >, <=, >=) require numeric operands
    ComparisonRequiresNumber {
        op: String,
        operand_desc: String,
        operand_type: ValueType,
        span: Span,
    },
    /// Arithmetic operators (+, -, *, /, %) require numeric operands
    ArithmeticRequiresNumber {
        op: BinaryOp,
        operand_type: ValueType,
        span: Span,
    },
    /// Unary operators (negation) require numeric operands
    UnaryRequiresNumber {
        op: UnaryOp,
        operand_type: ValueType,
        span: Span,
    },
}

impl IntoDiagnostic for SemanticError {
    fn into_diagnostic(self, ctx: &DiagnosticContext) -> Diagnostic {
        match self {
            SemanticError::UndefinedVariable { name, span } => {
                let mut diag = Diagnostic::error(
                    format!("undefined variable '{}'", name),
                    span,
                    "not defined in this scope",
                );

                // Add "did you mean?" suggestion using fuzzy matching
                if let Some(similar) = ctx.find_similar_variable(&name) {
                    diag = diag.with_suggestion(
                        format!("did you mean '{}'?", similar),
                        span,
                        similar.to_string(),
                    );
                }

                diag
            }
            SemanticError::Shadowing {
                name,
                span,
                original,
            } => Diagnostic::error(
                format!("variable '{}' shadows previous declaration", name),
                span,
                "shadows previous declaration",
            )
            .with_secondary(original, "previously declared here")
            .with_note("Bobbin does not allow shadowing to prevent confusion in dialogue scripts"),
            SemanticError::AssignmentToExtern { name, span } => Diagnostic::error(
                format!("cannot assign to extern variable '{}'", name),
                span,
                "extern variables are read-only",
            )
            .with_note(
                "Extern variables are provided by the host game and cannot be modified by scripts",
            )
            .with_note("Use 'save' or 'temp' to declare a mutable variable instead"),
            SemanticError::TypeMismatch {
                left_name,
                left_type,
                right_desc,
                right_type,
                span,
            } => Diagnostic::error(
                format!(
                    "type mismatch in comparison: {} ({}) vs {} ({})",
                    left_name,
                    left_type.name(),
                    right_desc,
                    right_type.name()
                ),
                span,
                format!("cannot compare {} to {}", left_type.name(), right_type.name()),
            ),
            SemanticError::ComparisonRequiresNumber {
                op,
                operand_desc,
                operand_type,
                span,
            } => Diagnostic::error(
                format!(
                    "operator '{}' requires numeric operands, but {} is {}",
                    op,
                    operand_desc,
                    operand_type.name()
                ),
                span,
                format!("cannot compare {} values with '{}'", operand_type.name(), op),
            )
            .with_note("Comparison operators (<, >, <=, >=) only work with numbers")
            .with_note("Use == or != to compare strings and booleans"),
            SemanticError::ArithmeticRequiresNumber {
                op,
                operand_type,
                span,
            } => {
                let op_str = Resolver::op_to_string(op);
                Diagnostic::error(
                    format!(
                        "operator '{}' requires numeric operands, got {}",
                        op_str,
                        operand_type.name()
                    ),
                    span,
                    format!("expected number, got {}", operand_type.name()),
                )
                .with_note("Arithmetic operators (+, -, *, /, %) only work with numbers")
            }
            SemanticError::UnaryRequiresNumber { op, operand_type, span } => {
                let op_str = match op {
                    UnaryOp::Negate => "-",
                };
                Diagnostic::error(
                    format!(
                        "operator '{}' requires numeric operand, got {}",
                        op_str,
                        operand_type.name()
                    ),
                    span,
                    format!("expected number, got {}", operand_type.name()),
                )
                .with_note("Negation operator only works with numbers")
            }
        }
    }
}

/// The kind of variable declaration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariableKind {
    Temp,
    Save,
    Extern,
}

/// Information about a variable declaration (for IDE features)
#[derive(Debug, Clone)]
pub struct VariableDeclaration {
    pub name: String,
    pub kind: VariableKind,
    pub span: Span,
}

/// Symbol table built during semantic analysis.
/// Maps each variable usage (by NodeId) to its storage location.
#[derive(Debug, Default)]
pub struct SymbolTable {
    /// Temp variable bindings: NodeId -> stack slot
    pub bindings: HashMap<NodeId, usize>,
    /// Save variable bindings: NodeId -> variable name
    pub save_bindings: HashMap<NodeId, String>,
    /// Extern variable bindings: NodeId -> variable name
    pub extern_bindings: HashMap<NodeId, String>,
    /// All variable declarations (for IDE features like autocomplete)
    pub declarations: Vec<VariableDeclaration>,
}

/// Information about a declared temp variable
#[derive(Debug)]
struct VarInfo {
    slot: usize,
    span: Span,
    value_type: ValueType,
}

/// Information about a declared save variable
#[derive(Debug)]
struct SaveVarInfo {
    span: Span,
    value_type: ValueType,
}

/// Information about a declared extern variable
#[derive(Debug)]
struct ExternVarInfo {
    span: Span, // for error messages (no slot - uses host state)
}

/// A lexical scope containing variable declarations
#[derive(Debug)]
struct Scope {
    variables: HashMap<String, VarInfo>,
    /// Slot count when this scope was created (for reclamation on pop)
    start_slot: usize,
}

#[derive(Debug)]
pub struct Resolver<'a> {
    ast: &'a Script,
    /// Temp variable scopes (block-scoped)
    scopes: Vec<Scope>,
    /// Save variables (file-global)
    save_vars: HashMap<String, SaveVarInfo>,
    /// Extern variables (file-global, read-only)
    extern_vars: HashMap<String, ExternVarInfo>,
    next_slot: usize,
    /// Temp variable bindings: NodeId -> slot
    bindings: HashMap<NodeId, usize>,
    /// Save variable bindings: NodeId -> name
    save_bindings: HashMap<NodeId, String>,
    /// Extern variable bindings: NodeId -> name
    extern_bindings: HashMap<NodeId, String>,
    /// All variable declarations (for IDE features)
    declarations: Vec<VariableDeclaration>,
    errors: Vec<SemanticError>,
}

impl<'a> Resolver<'a> {
    pub fn new(ast: &'a Script) -> Self {
        Self {
            ast,
            scopes: vec![Scope {
                variables: HashMap::new(),
                start_slot: 0,
            }], // Start with global scope
            save_vars: HashMap::new(),
            extern_vars: HashMap::new(),
            next_slot: 0,
            bindings: HashMap::new(),
            save_bindings: HashMap::new(),
            extern_bindings: HashMap::new(),
            declarations: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Analyze the AST and build a symbol table.
    ///
    /// Returns a tuple of:
    /// - Result with SymbolTable on success, or errors on failure
    /// - All declarations found (returned even on error for IDE features)
    /// - Known variable names (for "did you mean?" suggestions)
    pub fn analyze(
        mut self,
    ) -> (
        Result<SymbolTable, Vec<SemanticError>>,
        Vec<VariableDeclaration>,
        Vec<String>,
    ) {
        // Walk the AST
        for stmt in &self.ast.statements {
            self.resolve_stmt(stmt);
        }

        let declarations = self.declarations.clone();
        let known_vars = self.known_variables();

        if self.errors.is_empty() {
            (
                Ok(SymbolTable {
                    bindings: self.bindings,
                    save_bindings: self.save_bindings,
                    extern_bindings: self.extern_bindings,
                    declarations: self.declarations,
                }),
                declarations,
                known_vars,
            )
        } else {
            (Err(self.errors), declarations, known_vars)
        }
    }

    /// Get all known variable names for "did you mean?" suggestions.
    fn known_variables(&self) -> Vec<String> {
        let mut vars = Vec::new();

        // Collect temp variables from all scopes
        for scope in &self.scopes {
            vars.extend(scope.variables.keys().cloned());
        }

        // Collect save variables
        vars.extend(self.save_vars.keys().cloned());

        // Collect extern variables
        vars.extend(self.extern_vars.keys().cloned());

        vars
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::TempDecl(VarBindingData { id, name, value, span }) => {
                let value_type = ValueType::from_literal(value);
                self.declare_temp(*id, name, *span, value_type);
            }
            Stmt::SaveDecl(VarBindingData { id, name, value, span }) => {
                let value_type = ValueType::from_literal(value);
                self.declare_save(*id, name, *span, value_type);
            }
            Stmt::ExternDecl(ExternDeclData { id, name, span }) => {
                self.declare_extern(*id, name, *span);
            }
            Stmt::Assignment(VarBindingData { id, name, span, .. }) => {
                self.resolve_reference(*id, name, *span, true); // for_write = true
            }
            Stmt::Line { parts, .. } => {
                self.resolve_text_parts(parts);
            }
            Stmt::ChoiceSet { choices } => {
                // Resolve variable references in choice text
                for choice in choices {
                    self.resolve_text_parts(&choice.parts);
                }
                // Each choice branch gets its own scope
                for choice in choices {
                    self.resolve_choice_branch(choice);
                }
            }
        }
    }

    fn resolve_choice_branch(&mut self, choice: &Choice) {
        self.push_scope();
        for stmt in &choice.nested {
            self.resolve_stmt(stmt);
        }
        self.pop_scope();
    }

    fn resolve_text_parts(&mut self, parts: &[TextPart]) {
        for part in parts {
            match part {
                TextPart::Literal { .. } => {
                    // Literal text has no variables to resolve
                }
                TextPart::Expr { expr, .. } => {
                    self.resolve_expr(expr);
                }
            }
        }
    }

    /// Resolve variable references within an expression.
    fn resolve_expr(&mut self, expr: &crate::ast::Expr) {
        use crate::ast::Expr;
        match expr {
            Expr::Literal { .. } => {
                // Literals have no variables to resolve
            }
            Expr::VarRef { id, name, span } => {
                self.resolve_reference(*id, name, *span, false);
            }
            Expr::Unary { op, expr: inner, span, .. } => {
                self.resolve_expr(inner);
                self.check_unary_type(inner, *op, *span);
            }
            Expr::Binary { left, right, op, span, .. } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
                self.check_expr_types(left, right, *op, *span);
            }
        }
    }

    /// Get the type of an expression (for type checking).
    fn expr_type(&self, expr: &crate::ast::Expr) -> Option<ValueType> {
        use crate::ast::Expr;
        match expr {
            Expr::Literal { value, .. } => Some(ValueType::from_literal(value)),
            Expr::VarRef { name, .. } => self.lookup_type(name),
            Expr::Unary { op: UnaryOp::Negate, .. } => Some(ValueType::Number),
            Expr::Binary { op, .. } => match op {
                // Comparison operators return Bool
                BinaryOp::Equal
                | BinaryOp::NotEqual
                | BinaryOp::Less
                | BinaryOp::LessEqual
                | BinaryOp::Greater
                | BinaryOp::GreaterEqual => Some(ValueType::Bool),
                // Arithmetic operators return Number
                BinaryOp::Add
                | BinaryOp::Subtract
                | BinaryOp::Multiply
                | BinaryOp::Divide
                | BinaryOp::Modulo => Some(ValueType::Number),
            },
        }
    }

    /// Check that both operands of an expression comparison have compatible types.
    fn check_expr_types(
        &mut self,
        left: &crate::ast::Expr,
        right: &crate::ast::Expr,
        op: BinaryOp,
        span: Span,
    ) {
        let left_type = match self.expr_type(left) {
            Some(t) => t,
            None => return, // Extern or undefined - skip type checking
        };

        let right_type = match self.expr_type(right) {
            Some(t) => t,
            None => return, // Extern or undefined - skip type checking
        };

        // Arithmetic operators (+, -, *, /, %) require numbers
        if matches!(
            op,
            BinaryOp::Add
                | BinaryOp::Subtract
                | BinaryOp::Multiply
                | BinaryOp::Divide
                | BinaryOp::Modulo
        ) {
            if left_type != ValueType::Number {
                self.errors.push(SemanticError::ArithmeticRequiresNumber {
                    op,
                    operand_type: left_type,
                    span: left.span(),
                });
            }
            if right_type != ValueType::Number {
                self.errors.push(SemanticError::ArithmeticRequiresNumber {
                    op,
                    operand_type: right_type,
                    span: right.span(),
                });
            }
            return;
        }

        // Ordering comparisons (<, >, <=, >=) require numbers
        if matches!(
            op,
            BinaryOp::Less | BinaryOp::LessEqual | BinaryOp::Greater | BinaryOp::GreaterEqual
        ) {
            let op_str = Self::op_to_string(op);
            if left_type != ValueType::Number {
                self.errors.push(SemanticError::ComparisonRequiresNumber {
                    op: op_str.clone(),
                    operand_desc: self.describe_expr(left),
                    operand_type: left_type,
                    span: left.span(),
                });
            }
            if right_type != ValueType::Number {
                self.errors.push(SemanticError::ComparisonRequiresNumber {
                    op: op_str,
                    operand_desc: self.describe_expr(right),
                    operand_type: right_type,
                    span: right.span(),
                });
            }
            return;
        }

        // Equality comparisons (==, !=) require same type
        if left_type != right_type {
            self.errors.push(SemanticError::TypeMismatch {
                left_name: self.describe_expr(left),
                left_type,
                right_desc: self.describe_expr(right),
                right_type,
                span,
            });
        }
    }

    /// Convert a BinaryOp to its string representation for error messages.
    fn op_to_string(op: BinaryOp) -> String {
        match op {
            BinaryOp::Equal => "==".to_string(),
            BinaryOp::NotEqual => "!=".to_string(),
            BinaryOp::Less => "<".to_string(),
            BinaryOp::LessEqual => "<=".to_string(),
            BinaryOp::Greater => ">".to_string(),
            BinaryOp::GreaterEqual => ">=".to_string(),
            BinaryOp::Add => "+".to_string(),
            BinaryOp::Subtract => "-".to_string(),
            BinaryOp::Multiply => "*".to_string(),
            BinaryOp::Divide => "/".to_string(),
            BinaryOp::Modulo => "%".to_string(),
        }
    }

    /// Describe an expression for error messages.
    fn describe_expr(&self, expr: &crate::ast::Expr) -> String {
        use crate::ast::Expr;
        match expr {
            Expr::Literal { value, .. } => format!("literal {:?}", value),
            Expr::VarRef { name, .. } => name.clone(),
            Expr::Unary { .. } => "unary expression".to_string(),
            Expr::Binary { op, .. } => match op {
                BinaryOp::Equal
                | BinaryOp::NotEqual
                | BinaryOp::Less
                | BinaryOp::LessEqual
                | BinaryOp::Greater
                | BinaryOp::GreaterEqual => "comparison".to_string(),
                BinaryOp::Add
                | BinaryOp::Subtract
                | BinaryOp::Multiply
                | BinaryOp::Divide
                | BinaryOp::Modulo => "arithmetic expression".to_string(),
            },
        }
    }

    /// Check that a unary operator has a compatible operand type.
    fn check_unary_type(&mut self, expr: &crate::ast::Expr, op: UnaryOp, span: Span) {
        let operand_type = match self.expr_type(expr) {
            Some(t) => t,
            None => return, // Extern or undefined - skip type checking
        };

        match op {
            UnaryOp::Negate => {
                if operand_type != ValueType::Number {
                    self.errors.push(SemanticError::UnaryRequiresNumber {
                        op,
                        operand_type,
                        span,
                    });
                }
            }
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(Scope {
            variables: HashMap::new(),
            start_slot: self.next_slot,
        });
    }

    fn pop_scope(&mut self) {
        if let Some(scope) = self.scopes.pop() {
            // Reclaim slots for sibling scope reuse
            self.next_slot = scope.start_slot;
        }
    }

    /// Check if a variable name conflicts with save or extern variables.
    /// Returns the span of the conflicting declaration, if any.
    fn find_global_conflict(&self, name: &str) -> Option<Span> {
        if let Some(info) = self.save_vars.get(name) {
            return Some(info.span);
        }
        if let Some(info) = self.extern_vars.get(name) {
            return Some(info.span);
        }
        None
    }

    /// Check if a variable name conflicts with any temp variable in the given scopes.
    /// Returns the span of the conflicting declaration, if any.
    fn find_temp_conflict<'b>(
        &self,
        name: &str,
        scopes: impl Iterator<Item = &'b Scope>,
    ) -> Option<Span> {
        for scope in scopes {
            if let Some(info) = scope.variables.get(name) {
                return Some(info.span);
            }
        }
        None
    }

    /// Declare a temp variable in the current (innermost) scope
    fn declare_temp(&mut self, id: NodeId, name: &str, span: Span, value_type: ValueType) {
        // Check for conflict with save/extern variables (file-global)
        if let Some(original) = self.find_global_conflict(name) {
            self.errors.push(SemanticError::Shadowing {
                name: name.to_string(),
                span,
                original,
            });
            return;
        }

        // Check for shadowing - search outer scopes (skip current)
        if let Some(original) = self.find_temp_conflict(name, self.scopes.iter().rev().skip(1)) {
            self.errors.push(SemanticError::Shadowing {
                name: name.to_string(),
                span,
                original,
            });
            return;
        }

        // Check current scope for redeclaration
        let current_scope = self.scopes.last_mut().unwrap();
        if let Some(var_info) = current_scope.variables.get(name) {
            self.errors.push(SemanticError::Shadowing {
                name: name.to_string(),
                span,
                original: var_info.span,
            });
            return;
        }

        // Assign slot
        let slot = self.next_slot;
        self.next_slot += 1;

        // Record in current scope
        current_scope
            .variables
            .insert(name.to_string(), VarInfo { slot, span, value_type });

        // Record binding for this declaration
        self.bindings.insert(id, slot);

        // Record declaration for IDE features
        self.declarations.push(VariableDeclaration {
            name: name.to_string(),
            kind: VariableKind::Temp,
            span,
        });
    }

    /// Declare a save variable (file-global, uses external storage)
    fn declare_save(&mut self, id: NodeId, name: &str, span: Span, value_type: ValueType) {
        // Check for conflict with save/extern variables (file-global)
        if let Some(original) = self.find_global_conflict(name) {
            self.errors.push(SemanticError::Shadowing {
                name: name.to_string(),
                span,
                original,
            });
            return;
        }

        // Check for conflict with any temp variable in any scope
        if let Some(original) = self.find_temp_conflict(name, self.scopes.iter()) {
            self.errors.push(SemanticError::Shadowing {
                name: name.to_string(),
                span,
                original,
            });
            return;
        }

        // Register the save variable (file-global)
        self.save_vars
            .insert(name.to_string(), SaveVarInfo { span, value_type });

        // Record binding for this declaration
        self.save_bindings.insert(id, name.to_string());

        // Record declaration for IDE features
        self.declarations.push(VariableDeclaration {
            name: name.to_string(),
            kind: VariableKind::Save,
            span,
        });
    }

    /// Declare an extern variable (file-global, read-only, host-provided)
    fn declare_extern(&mut self, _id: NodeId, name: &str, span: Span) {
        // Check for conflict with save/extern variables (file-global)
        if let Some(original) = self.find_global_conflict(name) {
            self.errors.push(SemanticError::Shadowing {
                name: name.to_string(),
                span,
                original,
            });
            return;
        }

        // Check for conflict with any temp variable in any scope
        if let Some(original) = self.find_temp_conflict(name, self.scopes.iter()) {
            self.errors.push(SemanticError::Shadowing {
                name: name.to_string(),
                span,
                original,
            });
            return;
        }

        // Register the extern variable (file-global)
        // Note: No binding recorded for the declaration itself - only for references
        self.extern_vars
            .insert(name.to_string(), ExternVarInfo { span });

        // Record declaration for IDE features
        self.declarations.push(VariableDeclaration {
            name: name.to_string(),
            kind: VariableKind::Extern,
            span,
        });
    }

    /// Resolve a variable reference - search temp scopes, save variables, then extern variables.
    /// If for_write is true, this is an assignment target and extern variables are disallowed.
    fn resolve_reference(&mut self, id: NodeId, name: &str, span: Span, for_write: bool) {
        // Check temp scopes first (innermost to outermost)
        for scope in self.scopes.iter().rev() {
            if let Some(var_info) = scope.variables.get(name) {
                // Record binding for this reference
                self.bindings.insert(id, var_info.slot);
                return;
            }
        }

        // Check save variables (file-global)
        if self.save_vars.contains_key(name) {
            self.save_bindings.insert(id, name.to_string());
            return;
        }

        // Check extern variables (file-global, read-only)
        if self.extern_vars.contains_key(name) {
            if for_write {
                self.errors.push(SemanticError::AssignmentToExtern {
                    name: name.to_string(),
                    span,
                });
                return;
            }
            self.extern_bindings.insert(id, name.to_string());
            return;
        }

        // Not found in any scope
        self.errors.push(SemanticError::UndefinedVariable {
            name: name.to_string(),
            span,
        });
    }

    /// Look up the type of a declared variable.
    /// Returns None for extern variables (unknown type) or undefined variables.
    fn lookup_type(&self, name: &str) -> Option<ValueType> {
        // Check temp scopes (innermost to outermost)
        for scope in self.scopes.iter().rev() {
            if let Some(var_info) = scope.variables.get(name) {
                return Some(var_info.value_type);
            }
        }

        // Check save variables
        if let Some(save_info) = self.save_vars.get(name) {
            return Some(save_info.value_type);
        }

        // Extern variables have no compile-time type
        // Undefined variables also return None (error already reported by resolve_reference)
        None
    }

}
