//! Variable storage interface for dialogue globals.

use std::collections::HashMap;

use crate::Value;

/// Storage interface for dialogue globals (`save` variables).
///
/// This trait defines the contract for persistent variable storage that
/// survives save/load cycles. The game provides an implementation that
/// integrates with its save system.
pub trait VariableStorage {
    /// Get the current value of a dialogue global.
    fn get(&self, name: &str) -> Option<Value>;

    /// Set a dialogue global to a new value.
    fn set(&mut self, name: &str, value: Value);

    /// Initialize a variable only if it doesn't exist.
    ///
    /// This is used for `save` declarations to implement "default" semantics:
    /// - If the variable doesn't exist, create it with the given default value
    /// - If it already exists (from a previous save), leave it unchanged
    fn initialize_if_absent(&mut self, name: &str, default: Value);

    /// Check if a variable exists in storage.
    fn contains(&self, name: &str) -> bool;
}

/// In-memory implementation of [`VariableStorage`] for testing and simple use cases.
///
/// This implementation stores all variables in a `HashMap`. It's suitable for:
/// - Unit tests and integration tests
/// - Simple games that don't need persistence
/// - The default storage when no custom implementation is provided
///
/// For games that need save/load persistence, implement [`VariableStorage`]
/// with your game's save system.
#[derive(Debug, Default)]
pub struct MemoryStorage {
    values: HashMap<String, Value>,
}

impl MemoryStorage {
    /// Create a new empty storage.
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
