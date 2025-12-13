use crate::scanner::Span;

#[derive(Debug, Clone)]
pub struct Script {
    pub statements: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Line { text: String, span: Span },
}
