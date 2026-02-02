use std::sync::Arc;

use bobbin_syntax::{Diagnostic, DiagnosticContext, IntoDiagnostic, Severity};

use crate::chunk::{Chunk, Instruction, Value};
use crate::commands::{CommandError, CommandHandler};
use crate::storage::{HostState, VariableStorage};

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
    /// Division or modulo by zero
    DivisionByZero,
    /// Type mismatch at runtime (semantic analysis should prevent this, but fail explicitly)
    TypeMismatch { expected: &'static str, got: &'static str },
    /// A command invocation failed.
    CommandFailed {
        /// The command that failed.
        command: String,
        /// The underlying error.
        error: CommandError,
    },
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
            RuntimeError::DivisionByZero => {
                write!(f, "division by zero")
            }
            RuntimeError::TypeMismatch { expected, got } => {
                write!(f, "type mismatch: expected {}, got {}", expected, got)
            }
            RuntimeError::CommandFailed { command, error } => {
                write!(f, "command '{}' failed: {}", command, error)
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
            RuntimeError::DivisionByZero => Diagnostic {
                severity: Severity::Error,
                message: "division by zero".to_string(),
                labels: vec![],
                notes: vec!["Division and modulo operations require a non-zero divisor".to_string()],
                suggestions: vec![],
            },
            RuntimeError::TypeMismatch { expected, got } => Diagnostic {
                severity: Severity::Error,
                message: format!("type mismatch: expected {}, got {}", expected, got),
                labels: vec![],
                notes: vec!["This is likely a compiler bug - semantic analysis should have caught this".to_string()],
                suggestions: vec![],
            },
            RuntimeError::CommandFailed { command, error } => Diagnostic {
                severity: Severity::Error,
                message: format!("command '{}' failed: {}", command, error),
                labels: vec![],
                notes: vec![
                    "Command handlers can fail for various reasons".to_string(),
                    "Check your command implementation for the specific error".to_string(),
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
    commands: Arc<dyn CommandHandler>,
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
    pub fn new(
        chunk: Chunk,
        storage: Arc<dyn VariableStorage>,
        host: Arc<dyn HostState>,
        commands: Arc<dyn CommandHandler>,
    ) -> Self {
        Self {
            chunk,
            ip: 0,
            stack: Vec::new(),
            storage,
            host,
            commands,
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
                Instruction::JumpIfFalse { target } => {
                    let value = self.stack.pop().expect("stack underflow: compiler bug");
                    match value {
                        Value::Bool(false) => {
                            self.ip = target; // Absolute target, not relative offset
                        }
                        Value::Bool(true) => {
                            // Condition true - continue to next instruction (no jump)
                        }
                        _ => {
                            return Err(RuntimeError::TypeMismatch {
                                expected: "bool",
                                got: value.type_name(),
                            });
                        }
                    }
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
                Instruction::Equal | Instruction::NotEqual => {
                    let b = self.stack.pop().expect("stack underflow: compiler bug");
                    let a = self.stack.pop().expect("stack underflow: compiler bug");
                    let equal = a == b;
                    let result = match instruction {
                        Instruction::Equal => equal,
                        Instruction::NotEqual => !equal,
                        _ => unreachable!(),
                    };
                    self.stack.push(Value::Bool(result));
                }
                Instruction::Less
                | Instruction::LessEqual
                | Instruction::Greater
                | Instruction::GreaterEqual => {
                    let b = self.stack.pop().expect("stack underflow: compiler bug");
                    let a = self.stack.pop().expect("stack underflow: compiler bug");

                    let result = match (&a, &b) {
                        (Value::Number(a_num), Value::Number(b_num)) => match instruction {
                            Instruction::Less => a_num < b_num,
                            Instruction::LessEqual => a_num <= b_num,
                            Instruction::Greater => a_num > b_num,
                            Instruction::GreaterEqual => a_num >= b_num,
                            _ => unreachable!(),
                        },
                        // Type-checked by resolver; defensive fallback
                        _ => false,
                    };
                    self.stack.push(Value::Bool(result));
                }
                Instruction::Add
                | Instruction::Subtract
                | Instruction::Multiply
                | Instruction::Divide
                | Instruction::Modulo => {
                    let b = self.stack.pop().expect("stack underflow: compiler bug");
                    let a = self.stack.pop().expect("stack underflow: compiler bug");

                    let result = match (&a, &b) {
                        (Value::Number(a_num), Value::Number(b_num)) => match instruction {
                            Instruction::Add => Ok(*a_num + *b_num),
                            Instruction::Subtract => Ok(*a_num - *b_num),
                            Instruction::Multiply => Ok(*a_num * *b_num),
                            Instruction::Divide => {
                                if *b_num == 0.0 {
                                    return Err(RuntimeError::DivisionByZero);
                                }
                                Ok(*a_num / *b_num)
                            }
                            Instruction::Modulo => {
                                if *b_num == 0.0 {
                                    return Err(RuntimeError::DivisionByZero);
                                }
                                Ok(*a_num % *b_num)
                            }
                            _ => unreachable!(),
                        },
                        // Type mismatch: semantic analysis should prevent this, but fail explicitly
                        _ => {
                            return Err(RuntimeError::TypeMismatch {
                                expected: "number",
                                got: a.type_name(),
                            });
                        }
                    }?;
                    self.stack.push(Value::Number(result));
                }
                Instruction::Negate => {
                    let value = self.stack.pop().expect("stack underflow: compiler bug");
                    match value {
                        Value::Number(n) => self.stack.push(Value::Number(-n)),
                        _ => {
                            return Err(RuntimeError::TypeMismatch {
                                expected: "number",
                                got: value.type_name(),
                            });
                        }
                    }
                }
                Instruction::And => {
                    // Stack: [..., left, right] -> [..., result]
                    let b = self.stack.pop().expect("stack underflow: compiler bug");
                    let a = self.stack.pop().expect("stack underflow: compiler bug");
                    match (&a, &b) {
                        (Value::Bool(a_val), Value::Bool(b_val)) => {
                            self.stack.push(Value::Bool(*a_val && *b_val));
                        }
                        _ => {
                            return Err(RuntimeError::TypeMismatch {
                                expected: "bool",
                                got: a.type_name(),
                            });
                        }
                    }
                }
                Instruction::Or => {
                    // Stack: [..., left, right] -> [..., result]
                    let b = self.stack.pop().expect("stack underflow: compiler bug");
                    let a = self.stack.pop().expect("stack underflow: compiler bug");
                    match (&a, &b) {
                        (Value::Bool(a_val), Value::Bool(b_val)) => {
                            self.stack.push(Value::Bool(*a_val || *b_val));
                        }
                        _ => {
                            return Err(RuntimeError::TypeMismatch {
                                expected: "bool",
                                got: a.type_name(),
                            });
                        }
                    }
                }
                Instruction::Not => {
                    // Stack: [..., operand] -> [..., result]
                    let value = self.stack.pop().expect("stack underflow: compiler bug");
                    match value {
                        Value::Bool(b) => self.stack.push(Value::Bool(!b)),
                        _ => {
                            return Err(RuntimeError::TypeMismatch {
                                expected: "bool",
                                got: value.type_name(),
                            });
                        }
                    }
                }
                Instruction::Command { name, arg_count } => {
                    // Pop arguments in reverse order (they were pushed left-to-right)
                    let mut args = Vec::with_capacity(arg_count as usize);
                    for _ in 0..arg_count {
                        args.push(self.stack.pop().expect("stack underflow: compiler bug"));
                    }
                    args.reverse(); // Restore correct argument order

                    // Invoke command handler
                    if let Err(error) = self.commands.invoke(&name, &args) {
                        return Err(RuntimeError::CommandFailed {
                            command: name.clone(),
                            error,
                        });
                    }
                    // Commands are fire-and-forget; no return value pushed
                }
                Instruction::Return => {
                    // Note: stack may have locals remaining, that's OK
                    return Ok(StepResult::Done);
                }
            }
        }
    }
}
