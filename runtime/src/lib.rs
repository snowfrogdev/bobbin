mod parser;
mod scanner;

#[derive(Default)]
pub struct Runtime {
    lines: Vec<String>,
    index: usize,
}

impl Runtime {
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
