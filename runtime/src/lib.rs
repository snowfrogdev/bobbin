use std::fmt;

use crate::compiler::{CompileError, Compiler};
use crate::parser::{ParseError, Parser};
use crate::resolver::{Resolver, SemanticError};
use crate::scanner::Scanner;
use crate::vm::{StepResult, VM};

pub use crate::vm::RuntimeError;
pub use crate::chunk::Value;
pub use crate::storage::VariableStorage;

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
        }
    }
}

pub struct Runtime {
    vm: VM,
    source: String,
    current_line: Option<String>,
    current_choices: Option<Vec<String>>,
    is_done: bool,
}

impl Runtime {
    pub fn new(script: &str) -> Result<Self, BobbinError> {
        let tokens = Scanner::new(script).tokens();
        let ast = Parser::new(tokens).parse()?;
        let symbols = Resolver::new(&ast).analyze()?;
        let chunk = Compiler::new(&ast, &symbols).compile()?;

        let mut runtime = Self {
            vm: VM::new(chunk),
            source: script.to_string(),
            current_line: None,
            current_choices: None,
            is_done: false,
        };
        runtime.step_vm();
        Ok(runtime)
    }

    pub fn current_line(&self) -> &str {
        self.current_line.as_deref().unwrap_or("")
    }
    pub fn current_choices(&self) -> &[String] {
        self.current_choices.as_deref().unwrap_or(&[])
    }

    pub fn advance(&mut self) {
        if !self.is_done {
            self.step_vm();
        }
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

    fn step_vm(&mut self) {
        let result = self.vm.step();
        self.handle_step_result(result);
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
