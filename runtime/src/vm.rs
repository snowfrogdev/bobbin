use crate::chunk::{Chunk, Instruction, Value};
use crate::diagnostic::{Diagnostic, DiagnosticContext, IntoDiagnostic, Severity};
use crate::storage::{HostState, VariableStorage};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum RuntimeError {
    /// select_and_continue called when VM is not at a ChoiceSet instruction
    NotAtChoice,
    /// Choice index out of bounds
    InvalidChoiceIndex { index: usize, count: usize },
    /// Save variable not found in storage (storage may be corrupted or cleared)
    MissingSaveVariable { name: String },
    /// Extern variable not found in host state
    MissingExternVariable { name: String },
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::NotAtChoice => {
                write!(
                    f,
                    "select_and_continue called but VM is not waiting for a choice"
                )
            }
            RuntimeError::InvalidChoiceIndex { index, count } => {
                write!(
                    f,
                    "choice index {} out of bounds (only {} choices)",
                    index, count
                )
            }
            RuntimeError::MissingSaveVariable { name } => {
                write!(f, "save variable '{}' not found in storage", name)
            }
            RuntimeError::MissingExternVariable { name } => {
                write!(f, "extern variable '{}' not found in host state", name)
            }
        }
    }
}

impl std::error::Error for RuntimeError {}

impl IntoDiagnostic for RuntimeError {
    fn into_diagnostic(self, _ctx: &DiagnosticContext) -> Diagnostic {
        // Runtime errors don't have source spans - they occur during execution.
        // We use empty labels rather than dummy spans to avoid misleading source highlighting.
        match self {
            RuntimeError::NotAtChoice => Diagnostic {
                severity: Severity::Error,
                message: "select_and_continue called but VM is not waiting for a choice".to_string(),
                labels: vec![],
                notes: vec!["This is an API usage error - check your game logic".to_string()],
                suggestions: vec![],
            },
            RuntimeError::InvalidChoiceIndex { index, count } => Diagnostic {
                severity: Severity::Error,
                message: format!(
                    "choice index {} out of bounds (only {} choices available)",
                    index, count
                ),
                labels: vec![],
                notes: vec!["Check that the choice index is within the valid range".to_string()],
                suggestions: vec![],
            },
            RuntimeError::MissingSaveVariable { name } => Diagnostic {
                severity: Severity::Error,
                message: format!("save variable '{}' not found in storage", name),
                labels: vec![],
                notes: vec![
                    "This may indicate corrupted or cleared save data".to_string(),
                    "Ensure the variable was declared with 'save' before use".to_string(),
                ],
                suggestions: vec![],
            },
            RuntimeError::MissingExternVariable { name } => Diagnostic {
                severity: Severity::Error,
                message: format!("extern variable '{}' not found in host state", name),
                labels: vec![],
                notes: vec![
                    "The host game must provide this variable before running the script".to_string(),
                    "Check that your game's HostState implementation returns a value for this variable".to_string(),
                ],
                suggestions: vec![],
            },
        }
    }
}

pub(crate) enum StepResult {
    Line(String),
    Choice(Vec<String>),
    Done,
}

pub struct VM {
    chunk: Chunk,
    ip: usize,
    stack: Vec<Value>,
    storage: Arc<dyn VariableStorage>,
    host: Arc<dyn HostState>,
}

impl std::fmt::Debug for VM {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VM")
            .field("chunk", &self.chunk)
            .field("ip", &self.ip)
            .field("stack", &self.stack)
            .finish_non_exhaustive()
    }
}

impl VM {
    pub fn new(chunk: Chunk, storage: Arc<dyn VariableStorage>, host: Arc<dyn HostState>) -> Self {
        Self {
            chunk,
            ip: 0,
            stack: Vec::new(),
            storage,
            host,
        }
    }

    /// Returns true if the next instruction (following jumps) is Return (no more content).
    pub(crate) fn is_at_end(&self) -> bool {
        let mut ip = self.ip;
        loop {
            match self.chunk.code.get(ip) {
                Some(Instruction::Return) | None => return true,
                Some(Instruction::Jump { target }) => ip = *target,
                Some(Instruction::ChoiceSet { .. }) => {
                    // Waiting for choice - there's more content after selection
                    return false;
                }
                _ => return false,
            }
        }
    }

    /// Continue execution after user selects a choice.
    /// Call this after `step()` returns `Choice`. The ip should be pointing at ChoiceSet.
    pub(crate) fn select_and_continue(&mut self, index: usize) -> Result<StepResult, RuntimeError> {
        // Read ChoiceSet to get targets
        let instruction = self.chunk.code[self.ip].clone();

        if let Instruction::ChoiceSet { count, targets } = instruction {
            if index >= count {
                return Err(RuntimeError::InvalidChoiceIndex { index, count });
            }
            self.ip += 1;
            self.ip = targets[index];
        } else {
            return Err(RuntimeError::NotAtChoice);
        }

        // Continue normal execution
        self.run()
    }

    /// Execute until we hit a pause point (Line, Choice) or Done.
    pub(crate) fn step(&mut self) -> Result<StepResult, RuntimeError> {
        self.run()
    }

    /// Core execution loop.
    fn run(&mut self) -> Result<StepResult, RuntimeError> {
        loop {
            let instruction = self.chunk.code[self.ip].clone();
            self.ip += 1;

            match instruction {
                Instruction::Constant { index } => {
                    let value = self.chunk.constants[index].clone();
                    self.stack.push(value);
                }
                Instruction::GetLocal { slot } => {
                    let value = self.stack[slot].clone();
                    self.stack.push(value);
                }
                Instruction::SetLocal { slot } => {
                    let value = self.stack.pop().expect("stack underflow: compiler bug");
                    self.stack[slot] = value;
                }
                Instruction::Concat { count } => {
                    // Pop `count` values and concatenate as strings
                    let start = self.stack.len() - count;
                    let mut result = String::new();
                    for i in start..self.stack.len() {
                        result.push_str(&self.stack[i].to_string_value());
                    }
                    self.stack.truncate(start);
                    self.stack.push(Value::String(result));
                }
                Instruction::Line => {
                    let value = self.stack.pop().expect("stack underflow: compiler bug");
                    let text = value.to_string_value();
                    return Ok(StepResult::Line(text));
                }
                Instruction::ChoiceSet { count, .. } => {
                    // Pop choice texts from stack
                    let mut choices = Vec::with_capacity(count);
                    for _ in 0..count {
                        let value = self.stack.pop().expect("stack underflow: compiler bug");
                        let text = value.to_string_value();
                        choices.push(text);
                    }
                    choices.reverse();
                    // Back up ip so select_and_continue can read ChoiceSet for targets
                    self.ip -= 1;
                    return Ok(StepResult::Choice(choices));
                }
                Instruction::Jump { target } => {
                    self.ip = target;
                }
                Instruction::InitStorage { name } => {
                    let value = self.stack.pop().expect("stack underflow: compiler bug");
                    self.storage.initialize_if_absent(&name, value);
                }
                Instruction::GetStorage { name } => match self.storage.get(&name) {
                    Some(value) => self.stack.push(value),
                    None => return Err(RuntimeError::MissingSaveVariable { name }),
                },
                Instruction::SetStorage { name } => {
                    let value = self.stack.pop().expect("stack underflow: compiler bug");
                    self.storage.set(&name, value);
                }
                Instruction::GetHost { name } => match self.host.lookup(&name) {
                    Some(value) => self.stack.push(value),
                    None => return Err(RuntimeError::MissingExternVariable { name }),
                },
                Instruction::Return => {
                    // Note: stack may have locals remaining, that's OK
                    return Ok(StepResult::Done);
                }
            }
        }
    }
}
