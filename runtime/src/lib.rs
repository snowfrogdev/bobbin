use std::fmt;
use std::sync::Arc;

use crate::compiler::{CompileError, Compiler};
use crate::vm::{StepResult, VM};

// Re-export syntax crate types for backward compatibility
pub use bobbin_syntax::{
    validate, AriadneRenderer, Diagnostic, DiagnosticContext, IntoDiagnostic,
    JaroWinklerMatcher, Label, LabelStyle, LexicalError, LineIndex, Matcher, ParseError, Parser,
    Renderer, Resolver, Scanner, SemanticError, Severity, SourcePosition, Span, Suggestion,
    SymbolTable, Token, TokenKind,
};
// Re-export local types
pub use crate::chunk::Value;
pub use crate::storage::{HostState, VariableStorage};
pub use crate::vm::RuntimeError;

// Keep diagnostic and token modules as public for backward compatibility
pub mod diagnostic {
    pub use bobbin_syntax::{
        AriadneRenderer, Diagnostic, DiagnosticContext, IntoDiagnostic, JaroWinklerMatcher, Label,
        LabelStyle, LineIndex, Matcher, Renderer, Severity, SourcePosition, Suggestion,
    };
}
pub mod token {
    pub use bobbin_syntax::{Span, Token, TokenKind};
}

mod chunk;
mod compiler;
mod storage;
mod vm;

#[derive(Debug, Clone)]
pub enum BobbinError {
    Parse(Vec<ParseError>),
    /// Semantic errors with the list of known variables for "did you mean?" suggestions.
    Semantic {
        errors: Vec<SemanticError>,
        known_variables: Vec<String>,
    },
    Compile(CompileError),
    Runtime(RuntimeError),
}

impl From<Vec<ParseError>> for BobbinError {
    fn from(errors: Vec<ParseError>) -> Self {
        BobbinError::Parse(errors)
    }
}

impl From<(Vec<SemanticError>, Vec<String>)> for BobbinError {
    fn from((errors, known_variables): (Vec<SemanticError>, Vec<String>)) -> Self {
        BobbinError::Semantic {
            errors,
            known_variables,
        }
    }
}

impl From<CompileError> for BobbinError {
    fn from(err: CompileError) -> Self {
        BobbinError::Compile(err)
    }
}

impl From<RuntimeError> for BobbinError {
    fn from(err: RuntimeError) -> Self {
        BobbinError::Runtime(err)
    }
}

impl fmt::Display for BobbinError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BobbinError::Parse(errors) => {
                write!(f, "{} parse error(s)", errors.len())
            }
            BobbinError::Semantic { errors, .. } => {
                write!(f, "{} semantic error(s)", errors.len())
            }
            BobbinError::Compile(err) => {
                write!(f, "compile error: {:?}", err)
            }
            BobbinError::Runtime(err) => {
                write!(f, "runtime error: {}", err)
            }
        }
    }
}

impl BobbinError {
    /// Convert this error into diagnostics for rendering (consuming version).
    pub fn into_diagnostics(self) -> Vec<Diagnostic> {
        match self {
            BobbinError::Parse(errors) => {
                let matcher = JaroWinklerMatcher::default();
                let ctx = DiagnosticContext::new(&[], &matcher);
                errors
                    .into_iter()
                    .map(|e| e.into_diagnostic(&ctx))
                    .collect()
            }
            BobbinError::Semantic {
                errors,
                known_variables,
            } => {
                let matcher = JaroWinklerMatcher::default();
                let ctx = DiagnosticContext::new(&known_variables, &matcher);
                errors
                    .into_iter()
                    .map(|e| e.into_diagnostic(&ctx))
                    .collect()
            }
            BobbinError::Compile(_err) => {
                // CompileError is currently empty - handle when populated
                vec![]
            }
            BobbinError::Runtime(err) => {
                let matcher = JaroWinklerMatcher::default();
                let ctx = DiagnosticContext::new(&[], &matcher);
                vec![err.into_diagnostic(&ctx)]
            }
        }
    }

    /// Convert this error into diagnostics for rendering (borrowing version).
    ///
    /// This is more efficient than `into_diagnostics()` when you need to retain the error,
    /// as it only clones individual errors rather than the entire `BobbinError`.
    pub fn to_diagnostics(&self) -> Vec<Diagnostic> {
        match self {
            BobbinError::Parse(errors) => {
                let matcher = JaroWinklerMatcher::default();
                let ctx = DiagnosticContext::new(&[], &matcher);
                errors
                    .iter()
                    .map(|e| e.clone().into_diagnostic(&ctx))
                    .collect()
            }
            BobbinError::Semantic {
                errors,
                known_variables,
            } => {
                let matcher = JaroWinklerMatcher::default();
                let ctx = DiagnosticContext::new(known_variables, &matcher);
                errors
                    .iter()
                    .map(|e| e.clone().into_diagnostic(&ctx))
                    .collect()
            }
            BobbinError::Compile(_err) => {
                // CompileError is currently empty - handle when populated
                vec![]
            }
            BobbinError::Runtime(err) => {
                let matcher = JaroWinklerMatcher::default();
                let ctx = DiagnosticContext::new(&[], &matcher);
                vec![err.clone().into_diagnostic(&ctx)]
            }
        }
    }

    /// Render this error with beautiful terminal output.
    ///
    /// This is a convenience method that converts to diagnostics and renders them.
    pub fn render(&self, source_id: &str, source: &str) -> String {
        let diagnostics = self.to_diagnostics();
        let renderer = AriadneRenderer::new();
        // AriadneRenderer normalizes line endings internally
        renderer.render_all(&diagnostics, source_id, source)
    }
}

pub struct Runtime {
    vm: VM,
    storage: Arc<dyn VariableStorage>,
    host: Arc<dyn HostState>,
    current_line: Option<String>,
    current_choices: Option<Vec<String>>,
    is_done: bool,
}

impl Runtime {
    /// Create a new runtime with the given storage and host state.
    ///
    /// Both the game and the runtime share ownership of storage and host via `Arc`.
    /// This design allows the game engine to read and write storage while the
    /// dialogue runtime operates on them.
    ///
    /// Storage implementations use interior mutability (e.g., `RwLock`) to handle
    /// concurrent reads and writes safely.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use std::sync::Arc;
    ///
    /// let storage = Arc::new(MemoryStorage::new());
    /// let host = Arc::new(EmptyHostState);
    /// let mut runtime = Runtime::new(script, Arc::clone(&storage), Arc::clone(&host))?;
    ///
    /// // Game can read/write storage anytime via its Arc:
    /// let value = storage.get("reputation");
    /// storage.set("quest_complete", Value::Bool(true));
    /// ```
    pub fn new(
        script: &str,
        storage: Arc<dyn VariableStorage>,
        host: Arc<dyn HostState>,
    ) -> Result<Self, BobbinError> {
        let tokens = Scanner::new(script).tokens();
        let ast = Parser::new(tokens).parse()?;
        let symbols = Resolver::new(&ast).analyze()?;
        let chunk = Compiler::new(&ast, &symbols).compile()?;

        let mut runtime = Self {
            vm: VM::new(chunk, Arc::clone(&storage), Arc::clone(&host)),
            storage,
            host,
            current_line: None,
            current_choices: None,
            is_done: false,
        };
        runtime.step_vm()?;
        Ok(runtime)
    }

    /// Get a reference to the storage for external access.
    pub fn storage(&self) -> &Arc<dyn VariableStorage> {
        &self.storage
    }

    /// Get a reference to the host state for external access.
    pub fn host(&self) -> &Arc<dyn HostState> {
        &self.host
    }

    pub fn current_line(&self) -> &str {
        self.current_line.as_deref().unwrap_or("")
    }

    pub fn current_choices(&self) -> &[String] {
        self.current_choices.as_deref().unwrap_or(&[])
    }

    /// Advance to the next line of dialogue.
    ///
    /// Returns an error if a runtime error occurs (e.g., missing save variable).
    pub fn advance(&mut self) -> Result<(), RuntimeError> {
        if !self.is_done {
            self.step_vm()?;
        }
        Ok(())
    }

    pub fn has_more(&self) -> bool {
        !self.is_done
    }

    pub fn is_waiting_for_choice(&self) -> bool {
        self.current_choices.is_some()
    }

    pub fn select_choice(&mut self, index: usize) -> Result<(), RuntimeError> {
        if self.current_choices.is_some() {
            self.current_choices = None;
            let result = self.vm.select_and_continue(index)?;
            self.handle_step_result(result);
        }
        Ok(())
    }

    fn step_vm(&mut self) -> Result<(), RuntimeError> {
        let result = self.vm.step()?;
        self.handle_step_result(result);
        Ok(())
    }

    fn handle_step_result(&mut self, result: StepResult) {
        match result {
            StepResult::Line(text) => {
                self.current_line = Some(text);
                // Check if this was the last line (no more content after this)
                self.is_done = self.vm.is_at_end();
            }
            StepResult::Choice(choices) => {
                self.current_line = None;
                self.current_choices = Some(choices);
            }
            StepResult::Done => {
                self.current_line = None;
                self.is_done = true;
            }
        }
    }
}
