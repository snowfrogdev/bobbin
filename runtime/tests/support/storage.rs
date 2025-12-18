//! In-memory storage implementation for testing.

use bobbin_runtime::{Value, VariableStorage};
use std::cell::RefCell;
use std::collections::HashMap;

/// In-memory implementation of [`VariableStorage`] for testing.
///
/// This implementation stores all variables in a `HashMap` wrapped in `RefCell`
/// for interior mutability. It's suitable for:
/// - Unit tests and integration tests
/// - Simple games that don't need persistence
///
/// For games that need save/load persistence, implement [`VariableStorage`]
/// with your game's save system. Use `RefCell` for single-threaded games
/// or `Mutex`/`RwLock` for multi-threaded scenarios.
#[derive(Debug, Default)]
pub struct MemoryStorage {
    values: RefCell<HashMap<String, Value>>,
}

impl MemoryStorage {
    /// Create a new empty storage.
    pub fn new() -> Self {
        Self::default()
    }
}

impl VariableStorage for MemoryStorage {
    fn get(&self, name: &str) -> Option<Value> {
        self.values.borrow().get(name).cloned()
    }

    fn set(&self, name: &str, value: Value) {
        self.values.borrow_mut().insert(name.to_string(), value);
    }

    fn initialize_if_absent(&self, name: &str, default: Value) {
        self.values
            .borrow_mut()
            .entry(name.to_string())
            .or_insert(default);
    }

    fn contains(&self, name: &str) -> bool {
        self.values.borrow().contains_key(name)
    }
}
