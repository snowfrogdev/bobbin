use crate::ast::{Script, Stmt};
use crate::chunk::{Chunk, Instruction, Value};
use crate::resolver::SymbolTable;

#[derive(Debug)]
pub enum CompileError {}

#[derive(Debug)]
pub struct Compiler {
    ast: Script,
    chunk: Chunk,
    #[allow(dead_code)]
    symbols: SymbolTable,
}

impl Compiler {
    pub fn new(ast: Script, symbols: SymbolTable) -> Self {
        Self {
            ast,
            chunk: Chunk::new(),
            symbols,
        }
    }

    pub fn compile(mut self) -> Result<Chunk, CompileError> {
        let statements = std::mem::take(&mut self.ast.statements);
        for stmt in &statements {
            self.compile_stmt(stmt);
        }

        self.chunk.emit(Instruction::Return, 0);
        Ok(self.chunk)
    }

    fn compile_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Line { text, span } => {
                let index = self.chunk.add_constant(Value::String(text.clone()));
                self.chunk.emit(Instruction::Constant { index }, span.start);
                self.chunk.emit(Instruction::Line, span.start);
            }
            Stmt::ChoiceSet { choices } => {
                let count = choices.len();
                let line = choices[0].span.start;

                // 1. Emit constants for all choice texts
                for choice in choices {
                    let index = self.chunk.add_constant(Value::String(choice.text.clone()));
                    self.chunk
                        .emit(Instruction::Constant { index }, choice.span.start);
                }

                // 2. Emit ChoiceSet with placeholder targets (VM pauses here)
                let choice_set_offset = self.chunk.current_offset();
                self.chunk.emit(
                    Instruction::ChoiceSet {
                        count,
                        targets: vec![0; count],
                    },
                    line,
                );

                // 3. Emit nested code for each choice, collecting their start offsets
                let mut choice_targets = Vec::with_capacity(count);
                let mut jump_patches = Vec::new();

                for choice in choices {
                    // Record the start offset for this choice's nested code
                    choice_targets.push(self.chunk.current_offset());

                    // Emit nested statements
                    for nested_stmt in &choice.nested {
                        self.compile_stmt(nested_stmt);
                    }

                    // Emit Jump to gather point (placeholder target)
                    let jump_offset = self.chunk.current_offset();
                    self.chunk
                        .emit(Instruction::Jump { target: 0 }, choice.span.start);
                    jump_patches.push(jump_offset);
                }

                // 4. Gather point is here
                let gather_point = self.chunk.current_offset();

                // 5. Patch all Jump instructions to point to gather point
                for jump_offset in jump_patches {
                    self.chunk.patch_jump(jump_offset, gather_point);
                }

                // 6. Patch ChoiceSet with actual targets
                self.chunk
                    .patch_choice_targets(choice_set_offset, choice_targets);
            }
        }
    }
}
