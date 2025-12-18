use std::collections::HashMap;

use crate::ast::{Choice, ExternDeclData, NodeId, Script, Stmt, TextPart, VarBindingData};
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
    AssignmentToExtern {
        name: String,
        span: Span,
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
            SemanticError::AssignmentToExtern { name, span } => {
                let (line, col) = offset_to_position(source, span.start);
                format!(
                    "[{}:{}] cannot assign to extern variable '{}' (extern variables are read-only)",
                    line, col, name
                )
            }
        }
    }
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
}

/// Information about a declared temp variable
#[derive(Debug)]
struct VarInfo {
    slot: usize,
    span: Span, // for error messages
}

/// Information about a declared save variable
#[derive(Debug)]
struct SaveVarInfo {
    span: Span, // for error messages (no slot - uses external storage)
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
                save_bindings: self.save_bindings,
                extern_bindings: self.extern_bindings,
            })
        } else {
            Err(self.errors)
        }
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::TempDecl(VarBindingData { id, name, span, .. }) => {
                self.declare_temp(*id, name, *span);
            }
            Stmt::SaveDecl(VarBindingData { id, name, span, .. }) => {
                self.declare_save(*id, name, *span);
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
            if let TextPart::VarRef { id, name, span } = part {
                self.resolve_reference(*id, name, *span, false); // for_write = false
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

    /// Declare a temp variable in the current (innermost) scope
    fn declare_temp(&mut self, id: NodeId, name: &str, span: Span) {
        // Check for conflict with save variables (file-global)
        if let Some(save_info) = self.save_vars.get(name) {
            self.errors.push(SemanticError::Shadowing {
                name: name.to_string(),
                span,
                original: save_info.span,
            });
            return;
        }

        // Check for conflict with extern variables (file-global)
        if let Some(extern_info) = self.extern_vars.get(name) {
            self.errors.push(SemanticError::Shadowing {
                name: name.to_string(),
                span,
                original: extern_info.span,
            });
            return;
        }

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

        // Record in current scope
        current_scope
            .variables
            .insert(name.to_string(), VarInfo { slot, span });

        // Record binding for this declaration
        self.bindings.insert(id, slot);
    }

    /// Declare a save variable (file-global, uses external storage)
    fn declare_save(&mut self, id: NodeId, name: &str, span: Span) {
        // Check for conflict with existing save variable
        if let Some(save_info) = self.save_vars.get(name) {
            self.errors.push(SemanticError::Shadowing {
                name: name.to_string(),
                span,
                original: save_info.span,
            });
            return;
        }

        // Check for conflict with extern variables (file-global)
        if let Some(extern_info) = self.extern_vars.get(name) {
            self.errors.push(SemanticError::Shadowing {
                name: name.to_string(),
                span,
                original: extern_info.span,
            });
            return;
        }

        // Check for conflict with any temp variable in any scope
        for scope in &self.scopes {
            if let Some(var_info) = scope.variables.get(name) {
                self.errors.push(SemanticError::Shadowing {
                    name: name.to_string(),
                    span,
                    original: var_info.span,
                });
                return;
            }
        }

        // Register the save variable (file-global)
        self.save_vars
            .insert(name.to_string(), SaveVarInfo { span });

        // Record binding for this declaration
        self.save_bindings.insert(id, name.to_string());
    }

    /// Declare an extern variable (file-global, read-only, host-provided)
    fn declare_extern(&mut self, _id: NodeId, name: &str, span: Span) {
        // Check for conflict with existing extern variable (redeclaration)
        if let Some(extern_info) = self.extern_vars.get(name) {
            self.errors.push(SemanticError::Shadowing {
                name: name.to_string(),
                span,
                original: extern_info.span,
            });
            return;
        }

        // Check for conflict with save variables
        if let Some(save_info) = self.save_vars.get(name) {
            self.errors.push(SemanticError::Shadowing {
                name: name.to_string(),
                span,
                original: save_info.span,
            });
            return;
        }

        // Check for conflict with any temp variable in any scope
        for scope in &self.scopes {
            if let Some(var_info) = scope.variables.get(name) {
                self.errors.push(SemanticError::Shadowing {
                    name: name.to_string(),
                    span,
                    original: var_info.span,
                });
                return;
            }
        }

        // Register the extern variable (file-global)
        // Note: No binding recorded for the declaration itself - only for references
        self.extern_vars
            .insert(name.to_string(), ExternVarInfo { span });
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
}
