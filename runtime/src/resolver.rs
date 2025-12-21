use std::collections::HashMap;

use crate::ast::{Choice, ExternDeclData, NodeId, Script, Stmt, TextPart, VarBindingData};
use crate::diagnostic::{Diagnostic, DiagnosticContext, IntoDiagnostic};
use crate::token::Span;

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

    pub fn analyze(mut self) -> Result<SymbolTable, (Vec<SemanticError>, Vec<String>)> {
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
            let known_vars = self.known_variables();
            Err((self.errors, known_vars))
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
    fn declare_temp(&mut self, id: NodeId, name: &str, span: Span) {
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
            .insert(name.to_string(), VarInfo { slot, span });

        // Record binding for this declaration
        self.bindings.insert(id, slot);
    }

    /// Declare a save variable (file-global, uses external storage)
    fn declare_save(&mut self, id: NodeId, name: &str, span: Span) {
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
            .insert(name.to_string(), SaveVarInfo { span });

        // Record binding for this declaration
        self.save_bindings.insert(id, name.to_string());
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
