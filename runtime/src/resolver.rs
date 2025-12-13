use crate::ast::Script;

#[derive(Debug)]
pub enum SemanticError {}

/// Symbol table built during semantic analysis.
/// Future: scene names, labels, variable declarations, etc.
#[derive(Debug, Default)]
pub struct SymbolTable {
    // Empty for now — will hold scene/label/variable info
}

#[derive(Debug)]
pub struct Resolver<'a> {
    #[allow(dead_code)]
    ast: &'a Script,
}

impl<'a> Resolver<'a> {
    pub fn new(ast: &'a Script) -> Self {
        Self { ast }
    }

    pub fn analyze(self) -> Result<SymbolTable, SemanticError> {
        // No-op for now — just return empty symbol table
        // Future: walk AST, collect declarations, validate references
        Ok(SymbolTable::default())
    }
}
