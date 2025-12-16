#[derive(Debug, Clone)]
pub enum Instruction {
    Constant {
        index: usize,
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
    Return,
}

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
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
