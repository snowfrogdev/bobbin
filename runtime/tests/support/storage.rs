//! In-memory variable storage implementation for testing.

use bobbin_runtime::{Value, VariableStorage};
use std::collections::HashMap;

/// Simple in-memory storage for testing.
///
/// Each test should create a fresh instance to ensure isolation.
#[derive(Default)]
pub struct MemoryStorage {
    values: HashMap<String, Value>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self::default()
    }
}

impl VariableStorage for MemoryStorage {
    fn get(&self, name: &str) -> Option<Value> {
        self.values.get(name).cloned()
    }

    fn set(&mut self, name: &str, value: Value) {
        self.values.insert(name.to_string(), value);
    }

    fn initialize_if_absent(&mut self, name: &str, default: Value) {
        self.values.entry(name.to_string()).or_insert(default);
    }

    fn contains(&self, name: &str) -> bool {
        self.values.contains_key(name)
    }
}
