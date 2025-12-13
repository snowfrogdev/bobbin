use std::fmt;

use crate::compiler::{CompileError, Compiler};
use crate::parser::{ParseError, Parser};
use crate::resolver::{Resolver, SemanticError};
use crate::scanner::Scanner;
use crate::vm::{StepResult, VM};

mod ast;
mod chunk;
mod compiler;
mod parser;
mod resolver;
mod scanner;
mod vm;

#[derive(Debug)]
pub enum BobbinError {
    Parse(Vec<ParseError>),
    Semantic(SemanticError),
    Compile(CompileError),
}

impl From<Vec<ParseError>> for BobbinError {
    fn from(errors: Vec<ParseError>) -> Self {
        BobbinError::Parse(errors)
    }
}

impl From<SemanticError> for BobbinError {
    fn from(err: SemanticError) -> Self {
        BobbinError::Semantic(err)
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
            BobbinError::Semantic(err) => {
                write!(f, "semantic error: {:?}", err)
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
            BobbinError::Semantic(err) => format!("semantic error: {:?}", err),
            BobbinError::Compile(err) => format!("compile error: {:?}", err),
        }
    }
}

pub struct Runtime {
    vm: VM,
    source: String,
    current_line: Option<String>,
    is_done: bool,
}

impl Runtime {
    pub fn new(script: &str) -> Result<Self, BobbinError> {
        let tokens = Scanner::new(script).tokens();
        let ast = Parser::new(tokens).parse()?;
        let symbols = Resolver::new(&ast).analyze()?;
        let chunk = Compiler::new(ast, symbols).compile()?;

        let mut runtime = Self {
            vm: VM::new(chunk),
            source: script.to_string(),
            current_line: None,
            is_done: false,
        };
        runtime.step_vm();
        Ok(runtime)
    }

    pub fn current_line(&self) -> &str {
        self.current_line.as_deref().unwrap_or("")
    }

    pub fn advance(&mut self) {
        if !self.is_done {
            self.step_vm();
        }
    }

    pub fn has_more(&self) -> bool {
        !self.is_done
    }

    fn step_vm(&mut self) {
        match self.vm.step() {
            StepResult::Line(text) => {
                self.current_line = Some(text);
                // Check if this was the last line (no more content after this)
                self.is_done = self.vm.is_at_end();
            }
            StepResult::Done => {
                self.current_line = None;
                self.is_done = true;
            }
        }
    }
}
