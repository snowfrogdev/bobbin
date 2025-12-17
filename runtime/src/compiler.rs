use crate::ast::{Literal, NodeId, Script, Stmt, TextPart};
use crate::chunk::{Chunk, Instruction, Value};
use crate::resolver::SymbolTable;

#[derive(Debug)]
pub enum CompileError {}

#[derive(Debug)]
pub struct Compiler<'a> {
    ast: &'a Script,
    chunk: Chunk,
    symbols: &'a SymbolTable,
}

impl<'a> Compiler<'a> {
    pub fn new(ast: &'a Script, symbols: &'a SymbolTable) -> Self {
        Self {
            ast,
            chunk: Chunk::new(),
            symbols,
        }
    }

    pub fn compile(mut self) -> Result<Chunk, CompileError> {
        for stmt in &self.ast.statements {
            self.compile_stmt(stmt);
        }

        self.chunk.emit(Instruction::Return, 0);
        Ok(self.chunk)
    }

    /// Look up the stack slot for a NodeId. Panics if not found (resolver bug).
    fn get_slot(&self, id: NodeId) -> usize {
        *self
            .symbols
            .bindings
            .get(&id)
            .expect("binding not found: resolver bug")
    }

    fn compile_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::TempDecl {
                id, value, span, ..
            } => {
                // Push the initial value onto the stack.
                // The value will live at its assigned slot position.
                let slot = self.get_slot(*id);
                self.compile_literal(value, span.start);

                // For declarations, the value is already at the right position
                // if slots are assigned in declaration order. But to be safe
                // and support future reassignment, we could emit SetLocal.
                // For now, declarations just leave the value on stack at slot position.
                // This works because declarations happen in slot order.
                let _ = slot; // Slot is implicit from stack position
            }
            Stmt::Line { parts, span } => {
                self.compile_text_parts(parts, span.start);
                self.chunk.emit(Instruction::Line, span.start);
            }
            Stmt::ChoiceSet { choices } => {
                let count = choices.len();
                let line = choices[0].span.start;

                // 1. Emit code for all choice texts (may involve interpolation)
                for choice in choices {
                    self.compile_text_parts(&choice.parts, choice.span.start);
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

    /// Compile text parts (literals and variable references) onto the stack.
    /// If there's only one literal part, just push it.
    /// If there are multiple parts, push all and emit Concat.
    fn compile_text_parts(&mut self, parts: &[TextPart], line: usize) {
        if parts.is_empty() {
            // Empty text - push empty string
            let index = self.chunk.add_constant(Value::String(String::new()));
            self.chunk.emit(Instruction::Constant { index }, line);
            return;
        }

        // Optimization: single literal part, no concat needed
        if parts.len() == 1 {
            if let TextPart::Literal { text, .. } = &parts[0] {
                let index = self.chunk.add_constant(Value::String(text.clone()));
                self.chunk.emit(Instruction::Constant { index }, line);
                return;
            }
        }

        // Multiple parts or single var ref - push all and concat
        for part in parts {
            match part {
                TextPart::Literal { text, span } => {
                    let index = self.chunk.add_constant(Value::String(text.clone()));
                    self.chunk.emit(Instruction::Constant { index }, span.start);
                }
                TextPart::VarRef { id, span, .. } => {
                    let slot = self.get_slot(*id);
                    self.chunk.emit(Instruction::GetLocal { slot }, span.start);
                }
            }
        }

        // Concat if more than one part
        if parts.len() > 1 {
            self.chunk
                .emit(Instruction::Concat { count: parts.len() }, line);
        }
    }

    /// Compile a literal value and push onto stack.
    fn compile_literal(&mut self, literal: &Literal, line: usize) {
        let value = match literal {
            Literal::String(s) => Value::String(s.clone()),
            Literal::Number(n) => Value::Number(*n),
            Literal::Bool(b) => Value::Bool(*b),
        };
        let index = self.chunk.add_constant(value);
        self.chunk.emit(Instruction::Constant { index }, line);
    }
}
