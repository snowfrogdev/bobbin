use std::fmt;

use crate::compiler::{CompileError, Compiler};
use crate::parser::{ParseError, Parser};
use crate::resolver::{Resolver, SemanticError};
use crate::scanner::Scanner;
use crate::vm::{StepResult, VM};

pub use crate::vm::RuntimeError;
pub use crate::chunk::Value;
pub use crate::storage::{EmptyHostState, HostState, MemoryStorage, VariableStorage};

mod ast;
mod chunk;
mod compiler;
mod parser;
mod resolver;
mod scanner;
mod storage;
mod token;
mod vm;

#[derive(Debug)]
pub enum BobbinError {
    Parse(Vec<ParseError>),
    Semantic(Vec<SemanticError>),
    Compile(CompileError),
    Runtime(RuntimeError),
}

impl From<Vec<ParseError>> for BobbinError {
    fn from(errors: Vec<ParseError>) -> Self {
        BobbinError::Parse(errors)
    }
}

impl From<Vec<SemanticError>> for BobbinError {
    fn from(errors: Vec<SemanticError>) -> Self {
        BobbinError::Semantic(errors)
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
            BobbinError::Semantic(errors) => {
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
    /// Format error with source context (line:column).
    pub fn format_with_source(&self, source: &str) -> String {
        match self {
            BobbinError::Parse(errors) => errors
                .iter()
                .map(|e| e.format_with_source(source))
                .collect::<Vec<_>>()
                .join("\n"),
            BobbinError::Semantic(errors) => errors
                .iter()
                .map(|e| e.format_with_source(source))
                .collect::<Vec<_>>()
                .join("\n"),
            BobbinError::Compile(err) => format!("compile error: {:?}", err),
            BobbinError::Runtime(err) => format!("runtime error: {}", err),
        }
    }
}

pub struct Runtime {
    vm: VM,
    storage: Box<dyn VariableStorage>,
    host_state: Box<dyn HostState>,
    source: String,
    current_line: Option<String>,
    current_choices: Option<Vec<String>>,
    is_done: bool,
}

impl Runtime {
    /// Create a new runtime with default in-memory storage and no host state.
    ///
    /// This is suitable for testing and simple games that don't need persistence
    /// or host-provided variables.
    pub fn new(script: &str) -> Result<Self, BobbinError> {
        Self::with_storage_and_host(
            script,
            Box::new(MemoryStorage::new()),
            Box::new(EmptyHostState),
        )
    }

    /// Create a new runtime with custom storage and no host state.
    ///
    /// Use this when you need dialogue state to persist across save/load cycles.
    /// The game provides a [`VariableStorage`] implementation that integrates
    /// with its save system.
    pub fn with_storage(
        script: &str,
        storage: Box<dyn VariableStorage>,
    ) -> Result<Self, BobbinError> {
        Self::with_storage_and_host(script, storage, Box::new(EmptyHostState))
    }

    /// Create a new runtime with custom storage and host state.
    ///
    /// Use this when dialogue scripts need access to host-provided variables
    /// (declared with `extern` in Bobbin scripts) in addition to persistent storage.
    pub fn with_storage_and_host(
        script: &str,
        storage: Box<dyn VariableStorage>,
        host_state: Box<dyn HostState>,
    ) -> Result<Self, BobbinError> {
        let tokens = Scanner::new(script).tokens();
        let ast = Parser::new(tokens).parse()?;
        let symbols = Resolver::new(&ast).analyze()?;
        let chunk = Compiler::new(&ast, &symbols).compile()?;

        let mut runtime = Self {
            vm: VM::new(chunk),
            storage,
            host_state,
            source: script.to_string(),
            current_line: None,
            current_choices: None,
            is_done: false,
        };
        runtime.step_vm()?;
        Ok(runtime)
    }

    pub fn current_line(&self) -> &str {
        self.current_line.as_deref().unwrap_or("")
    }

    pub fn current_choices(&self) -> &[String] {
        self.current_choices.as_deref().unwrap_or(&[])
    }

    /// Access the storage for inspection (useful for testing and debugging).
    pub fn storage(&self) -> &dyn VariableStorage {
        &*self.storage
    }

    /// Access the host state for inspection (useful for testing and debugging).
    pub fn host_state(&self) -> &dyn HostState {
        &*self.host_state
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
            let result = self.vm.select_and_continue(index, &mut *self.storage)?;
            self.handle_step_result(result);
        }
        Ok(())
    }

    fn step_vm(&mut self) -> Result<(), RuntimeError> {
        let result = self.vm.step(&mut *self.storage)?;
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
