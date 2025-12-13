use crate::chunk::{Chunk, Instruction, Value};

pub(crate) enum StepResult {
    Line(String),
    Done,
}

#[derive(Debug)]
pub struct VM {
    chunk: Chunk,
    ip: usize,
    stack: Vec<Value>,
}

impl VM {
    pub fn new(chunk: Chunk) -> Self {
        Self {
            chunk,
            ip: 0,
            stack: Vec::new(),
        }
    }

    /// Returns true if the next instruction is Return (no more content).
    pub(crate) fn is_at_end(&self) -> bool {
        matches!(self.chunk.code.get(self.ip), Some(Instruction::Return) | None)
    }

    pub(crate) fn step(&mut self) -> StepResult {
        loop {
            let instruction = self.chunk.code[self.ip];
            self.ip += 1;

            match instruction {
                Instruction::Constant(index) => {
                    let value = self.chunk.constants[index].clone();
                    self.stack.push(value);
                }
                Instruction::Line => {
                    let value = self.stack.pop().expect("stack underflow: compiler bug");
                    let Value::String(text) = value;
                    return StepResult::Line(text);
                }
                Instruction::Return => {
                    debug_assert!(
                        self.stack.is_empty(),
                        "stack not empty at return: compiler bug"
                    );
                    return StepResult::Done;
                }
            }
        }
    }
}
