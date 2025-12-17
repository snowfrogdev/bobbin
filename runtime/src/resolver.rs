use std::collections::HashMap;

use crate::ast::{Choice, NodeId, Script, Stmt, TextPart};
use crate::scanner::offset_to_position;
use crate::token::Span;

#[derive(Debug)]
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
}

impl SemanticError {
    pub fn format_with_source(&self, source: &str) -> String {
        match self {
            SemanticError::UndefinedVariable { name, span } => {
                let (line, col) = offset_to_position(source, span.start);
                format!("[{}:{}] undefined variable: {}", line, col, name)
            }
            SemanticError::Shadowing {
                name,
                span,
                original,
            } => {
                let (line, col) = offset_to_position(source, span.start);
                let (orig_line, orig_col) = offset_to_position(source, original.start);
                format!(
                    "[{}:{}] variable '{}' shadows declaration at [{}:{}]",
                    line, col, name, orig_line, orig_col
                )
            }
        }
    }
}

/// Symbol table built during semantic analysis.
/// Maps each variable usage (by NodeId) to its stack slot index.
#[derive(Debug, Default)]
pub struct SymbolTable {
    /// Each declaration and reference NodeId -> stack slot
    pub bindings: HashMap<NodeId, usize>,
}

/// Information about a declared variable
#[derive(Debug)]
struct VarInfo {
    slot: usize,
    span: Span, // for error messages
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
    scopes: Vec<Scope>,
    next_slot: usize,
    max_slot: usize, // high water mark
    bindings: HashMap<NodeId, usize>,
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
            next_slot: 0,
            max_slot: 0,
            bindings: HashMap::new(),
            errors: Vec::new(),
        }
    }

    pub fn analyze(mut self) -> Result<SymbolTable, Vec<SemanticError>> {
        // Walk the AST
        for stmt in &self.ast.statements {
            self.resolve_stmt(stmt);
        }

        if self.errors.is_empty() {
            Ok(SymbolTable {
                bindings: self.bindings,
            })
        } else {
            Err(self.errors)
        }
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::TempDecl { id, name, span, .. } => {
                self.declare(*id, name, *span);
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
            if let TextPart::VarRef { id, name, span } = part {
                self.resolve_reference(*id, name, *span);
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

    /// Declare a variable in the current (innermost) scope
    fn declare(&mut self, id: NodeId, name: &str, span: Span) {
        // Check for shadowing - search outer scopes
        for scope in self.scopes.iter().rev().skip(1) {
            if let Some(var_info) = scope.variables.get(name) {
                self.errors.push(SemanticError::Shadowing {
                    name: name.to_string(),
                    span,
                    original: var_info.span,
                });
                return;
            }
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
        if self.next_slot > self.max_slot {
            self.max_slot = self.next_slot;
        }

        // Record in current scope
        current_scope
            .variables
            .insert(name.to_string(), VarInfo { slot, span });

        // Record binding for this declaration
        self.bindings.insert(id, slot);
    }

    /// Resolve a variable reference - search from innermost to outermost scope
    fn resolve_reference(&mut self, id: NodeId, name: &str, span: Span) {
        for scope in self.scopes.iter().rev() {
            if let Some(var_info) = scope.variables.get(name) {
                // Record binding for this reference
                self.bindings.insert(id, var_info.slot);
                return;
            }
        }
        // Not found in any scope
        self.errors.push(SemanticError::UndefinedVariable {
            name: name.to_string(),
            span,
        });
    }
}
