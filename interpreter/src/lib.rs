#[derive(Default)]
pub struct Interpreter {
    lines: Vec<String>,
    index: usize,
}

impl Interpreter {
    pub fn new() -> Self {
        Self { lines: Vec::new(), index: 0 }
    }

    pub fn load_content(&mut self, content: &str) {
        self.lines = content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|s| s.to_string())
            .collect();
        self.index = 0;
    }

    pub fn advance(&mut self) {
        if self.has_more() {
            self.index += 1;
        }
    }

    pub fn current_line(&self) -> &str {
        self.lines.get(self.index).map(|s| s.as_str()).unwrap_or("")
    }

    pub fn has_more(&self) -> bool {
        self.index + 1 < self.lines.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_flow() {
        let mut interp = Interpreter::new();
        interp.load_content("Line 1\nLine 2\nLine 3");
        assert_eq!(interp.current_line(), "Line 1");
        assert!(interp.has_more());
        interp.advance();
        assert_eq!(interp.current_line(), "Line 2");
        interp.advance();
        assert_eq!(interp.current_line(), "Line 3");
        assert!(!interp.has_more());
    }
}
