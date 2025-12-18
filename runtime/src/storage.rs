//! Variable storage interfaces for dialogue globals and host state.

use crate::Value;

/// Storage interface for dialogue globals (`save` variables).
///
/// This trait defines the contract for persistent variable storage that
/// survives save/load cycles. The game provides an implementation that
/// integrates with its save system.
///
/// # Interior Mutability
///
/// The mutating methods (`set`, `initialize_if_absent`) take `&self` rather than
/// `&mut self`. This enables both the game and the dialogue runtime to access
/// storage simultaneously through shared references.
///
/// Implementations should use interior mutability (e.g., `RefCell`, `Mutex`)
/// to handle the actual mutation.
///
/// # Example
///
/// ```rust
/// use std::cell::RefCell;
/// use std::collections::HashMap;
/// use bobbin_runtime::{Value, VariableStorage};
///
/// #[derive(Debug, Default)]
/// struct MemoryStorage {
///     values: RefCell<HashMap<String, Value>>,
/// }
///
/// impl VariableStorage for MemoryStorage {
///     fn get(&self, name: &str) -> Option<Value> {
///         self.values.borrow().get(name).cloned()
///     }
///
///     fn set(&self, name: &str, value: Value) {
///         self.values.borrow_mut().insert(name.to_string(), value);
///     }
///
///     fn initialize_if_absent(&self, name: &str, default: Value) {
///         self.values.borrow_mut().entry(name.to_string()).or_insert(default);
///     }
///
///     fn contains(&self, name: &str) -> bool {
///         self.values.borrow().contains_key(name)
///     }
/// }
/// ```
pub trait VariableStorage {
    /// Get the current value of a dialogue global.
    fn get(&self, name: &str) -> Option<Value>;

    /// Set a dialogue global to a new value.
    ///
    /// Takes `&self` to allow shared access. Implementations should use
    /// interior mutability (e.g., `RefCell`, `Mutex`).
    fn set(&self, name: &str, value: Value);

    /// Initialize a variable only if it doesn't exist.
    ///
    /// This is used for `save` declarations to implement "default" semantics:
    /// - If the variable doesn't exist, create it with the given default value
    /// - If it already exists (from a previous save), leave it unchanged
    ///
    /// Takes `&self` to allow shared access. Implementations should use
    /// interior mutability (e.g., `RefCell`, `Mutex`).
    fn initialize_if_absent(&self, name: &str, default: Value);

    /// Check if a variable exists in storage.
    fn contains(&self, name: &str) -> bool;
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
/// # Interior Mutability for Dynamic State
///
/// If the game needs to update host state while dialogue is running,
/// use interior mutability in your implementation:
///
/// ```rust
/// use std::cell::Cell;
/// use bobbin_runtime::{HostState, Value};
///
/// struct GameState {
///     player_health: Cell<i64>,
///     gold: Cell<i64>,
/// }
///
/// impl HostState for GameState {
///     fn lookup(&self, name: &str) -> Option<Value> {
///         match name {
///             "player_health" => Some(Value::Number(self.player_health.get() as f64)),
///             "gold" => Some(Value::Number(self.gold.get() as f64)),
///             _ => None,
///         }
///     }
/// }
///
/// // Game can update values anytime:
/// // game_state.player_health.set(50);
/// ```
pub trait HostState {
    /// Look up a host variable by name.
    ///
    /// Returns `Some(value)` if the variable exists, `None` otherwise.
    /// A `None` return will cause `RuntimeError::MissingExternVariable` at runtime.
    fn lookup(&self, name: &str) -> Option<Value>;
}
