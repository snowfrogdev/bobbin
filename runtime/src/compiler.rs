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
                self.chunk.emit(Instruction::Constant(index), span.start);
                self.chunk.emit(Instruction::Line, span.start);
            }
        }
    }
}
