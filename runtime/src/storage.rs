//! Variable storage interfaces for dialogue globals and host state.

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

/// Interface for host-provided variables (read-only from Bobbin's perspective).
///
/// The host application implements this trait to expose variables like
/// player health, gold, or other game state to dialogue scripts.
///
/// Variables accessed through this interface are declared with `extern`
/// in Bobbin scripts. They are read-only from the dialogue's perspective;
/// attempting to use `set` on an extern variable is a compile-time error.
///
/// # Example
///
/// ```rust
/// use bobbin_runtime::{HostState, Value};
///
/// struct GameState {
///     player_health: i64,
///     gold: i64,
/// }
///
/// impl HostState for GameState {
///     fn lookup(&self, name: &str) -> Option<Value> {
///         match name {
///             "player_health" => Some(Value::Number(self.player_health as f64)),
///             "gold" => Some(Value::Number(self.gold as f64)),
///             _ => None,
///         }
///     }
/// }
/// ```
pub trait HostState {
    /// Look up a host variable by name.
    ///
    /// Returns `Some(value)` if the variable exists, `None` otherwise.
    /// A `None` return will cause `RuntimeError::MissingExternVariable` at runtime.
    fn lookup(&self, name: &str) -> Option<Value>;
}

/// Empty host state that provides no variables.
///
/// This is the default implementation used when no host state is provided.
/// All lookups return `None`.
#[derive(Debug, Default, Clone, Copy)]
pub struct EmptyHostState;

impl HostState for EmptyHostState {
    fn lookup(&self, _name: &str) -> Option<Value> {
        None
    }
}
