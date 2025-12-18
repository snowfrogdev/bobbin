//! In-memory storage implementation for testing.

use bobbin_runtime::{Value, VariableStorage};
use std::collections::HashMap;
use std::sync::RwLock;

/// In-memory implementation of [`VariableStorage`] for testing.
///
/// This implementation stores all variables in a `HashMap` wrapped in `RwLock`
/// for thread-safe interior mutability. It's suitable for:
/// - Unit tests and integration tests
/// - Simple games that don't need persistence
/// - Both single-threaded and multi-threaded scenarios
///
/// For games that need save/load persistence, implement [`VariableStorage`]
/// with your game's save system.
#[derive(Debug, Default)]
pub struct MemoryStorage {
    values: RwLock<HashMap<String, Value>>,
}

impl MemoryStorage {
    /// Create a new empty storage.
    pub fn new() -> Self {
        Self::default()
    }
}

impl VariableStorage for MemoryStorage {
    fn get(&self, name: &str) -> Option<Value> {
        self.values.read().unwrap().get(name).cloned()
    }

    fn set(&self, name: &str, value: Value) {
        self.values.write().unwrap().insert(name.to_string(), value);
    }

    fn initialize_if_absent(&self, name: &str, default: Value) {
        self.values
            .write()
            .unwrap()
            .entry(name.to_string())
            .or_insert(default);
    }

    fn contains(&self, name: &str) -> bool {
        self.values.read().unwrap().contains_key(name)
    }
}
