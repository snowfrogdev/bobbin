#[derive(Debug, Clone)]
pub enum Instruction {
    Constant {
        index: usize,
    },
    /// Copy value from stack[slot] to top of stack.
    GetLocal {
        slot: usize,
    },
    /// Pop top of stack and write to stack[slot].
    SetLocal {
        slot: usize,
    },
    /// Pop `count` values, concatenate as strings, push result.
    Concat {
        count: usize,
    },
    Line,
    /// Present choices to the user. VM pauses for selection.
    /// On resume, jumps to targets[selected_index].
    ChoiceSet {
        count: usize,
        targets: Vec<usize>,
    },
    /// Unconditional jump to target instruction index.
    Jump {
        target: usize,
    },
    /// Initialize a save variable only if it doesn't exist in storage.
    /// Pops value from stack, calls storage.initialize_if_absent(name, value).
    InitStorage {
        name: String,
    },
    /// Read a save variable from storage and push onto stack.
    GetStorage {
        name: String,
    },
    /// Pop value from stack and write to save variable in storage.
    SetStorage {
        name: String,
    },
    /// Read a host variable via HostState and push onto stack.
    GetHost {
        name: String,
    },
    Return,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Number(f64),
    Bool(bool),
}

impl Value {
    /// Convert value to string representation for interpolation.
    pub fn to_string_value(&self) -> String {
        match self {
            Value::String(s) => s.clone(),
            Value::Number(n) => {
                // Format integers without decimal point
                if n.fract() == 0.0 {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            Value::Bool(b) => if *b { "true" } else { "false" }.to_string(),
        }
    }
}

#[derive(Debug)]
pub struct Chunk {
    pub code: Vec<Instruction>,
    pub constants: Vec<Value>,
    pub lines: Vec<usize>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
            lines: Vec::new(),
        }
    }

    pub fn emit(&mut self, instruction: Instruction, line: usize) {
        self.code.push(instruction);
        self.lines.push(line);
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    /// Returns the index where the next instruction will be emitted.
    pub fn current_offset(&self) -> usize {
        self.code.len()
    }

    /// Patch a Jump instruction at `offset` to jump to `target`.
    pub fn patch_jump(&mut self, offset: usize, target: usize) {
        if let Instruction::Jump { target: ref mut t } = self.code[offset] {
            *t = target;
        } else {
            panic!("patch_jump called on non-Jump instruction");
        }
    }

    /// Patch a ChoiceSet instruction's targets at `offset`.
    pub fn patch_choice_targets(&mut self, offset: usize, new_targets: Vec<usize>) {
        if let Instruction::ChoiceSet { targets, .. } = &mut self.code[offset] {
            *targets = new_targets;
        } else {
            panic!("patch_choice_targets called on non-ChoiceSet instruction");
        }
    }
}
