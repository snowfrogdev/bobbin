use crate::ast::{Literal, NodeId, Script, Stmt, TextPart, VarBindingData};
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

    /// Look up the save variable name for a NodeId. Returns None if not a save variable.
    fn get_save_name(&self, id: NodeId) -> Option<&str> {
        self.symbols.save_bindings.get(&id).map(|s| s.as_str())
    }

    /// Emit instruction to read a variable (temp or save) and push onto stack.
    fn emit_var_read(&mut self, id: NodeId, line: usize) {
        if let Some(name) = self.get_save_name(id) {
            self.chunk
                .emit(Instruction::GetStorage { name: name.to_string() }, line);
        } else {
            let slot = self.get_slot(id);
            self.chunk.emit(Instruction::GetLocal { slot }, line);
        }
    }

    /// Emit instruction to write a value (already on stack) to a variable (temp or save).
    fn emit_var_write(&mut self, id: NodeId, line: usize) {
        if let Some(name) = self.get_save_name(id) {
            self.chunk
                .emit(Instruction::SetStorage { name: name.to_string() }, line);
        } else {
            let slot = self.get_slot(id);
            self.chunk.emit(Instruction::SetLocal { slot }, line);
        }
    }

    fn compile_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::TempDecl(VarBindingData { value, span, .. }) => {
                // Push initial value onto stack.
                // The value lives at its assigned slot position (implicit from declaration order).
                self.compile_literal(value, span.start);
            }
            Stmt::SaveDecl(VarBindingData { name, value, span, .. }) => {
                // Push initial value onto stack, then emit InitStorage.
                // InitStorage uses "initialize if absent" semantics for save variables.
                self.compile_literal(value, span.start);
                self.chunk.emit(
                    Instruction::InitStorage { name: name.clone() },
                    span.start,
                );
            }
            Stmt::Assignment(VarBindingData { id, value, span, .. }) => {
                // Assignment modifies an existing variable (temp or save).
                // Push value, then emit appropriate write instruction.
                self.compile_literal(value, span.start);
                self.emit_var_write(*id, span.start);
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
                    self.emit_var_read(*id, span.start);
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
