//! Variable storage interfaces for dialogue globals and host state.

use crate::Value;

/// Storage interface for dialogue globals (`save` variables).
///
/// This trait defines the contract for persistent variable storage that
/// survives save/load cycles. The game provides an implementation that
/// integrates with its save system.
///
/// # Thread Safety
///
/// Implementations must be `Send + Sync` to allow the runtime to be used
/// across threads. Use thread-safe interior mutability (e.g., `RwLock`, `Mutex`)
/// for the actual mutation.
///
/// # Interior Mutability
///
/// The mutating methods (`set`, `initialize_if_absent`) take `&self` rather than
/// `&mut self`. This enables both the game and the dialogue runtime to access
/// storage simultaneously through shared references.
///
/// # Example
///
/// ```rust
/// use std::sync::RwLock;
/// use std::collections::HashMap;
/// use bobbin_runtime::{Value, VariableStorage};
///
/// #[derive(Debug, Default)]
/// struct MemoryStorage {
///     values: RwLock<HashMap<String, Value>>,
/// }
///
/// impl VariableStorage for MemoryStorage {
///     fn get(&self, name: &str) -> Option<Value> {
///         self.values.read().unwrap().get(name).cloned()
///     }
///
///     fn set(&self, name: &str, value: Value) {
///         self.values.write().unwrap().insert(name.to_string(), value);
///     }
///
///     fn initialize_if_absent(&self, name: &str, default: Value) {
///         self.values.write().unwrap().entry(name.to_string()).or_insert(default);
///     }
///
///     fn contains(&self, name: &str) -> bool {
///         self.values.read().unwrap().contains_key(name)
///     }
/// }
/// ```
pub trait VariableStorage: Send + Sync {
    /// Get the current value of a dialogue global.
    fn get(&self, name: &str) -> Option<Value>;

    /// Set a dialogue global to a new value.
    ///
    /// Takes `&self` to allow shared access. Implementations should use
    /// thread-safe interior mutability (e.g., `RwLock`, `Mutex`).
    fn set(&self, name: &str, value: Value);

    /// Initialize a variable only if it doesn't exist.
    ///
    /// This is used for `save` declarations to implement "default" semantics:
    /// - If the variable doesn't exist, create it with the given default value
    /// - If it already exists (from a previous save), leave it unchanged
    ///
    /// Takes `&self` to allow shared access. Implementations should use
    /// thread-safe interior mutability (e.g., `RwLock`, `Mutex`).
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
/// # Thread Safety
///
/// Implementations must be `Send + Sync` to allow the runtime to be used
/// across threads. Use thread-safe interior mutability if values can change.
///
/// # Interior Mutability for Dynamic State
///
/// If the game needs to update host state while dialogue is running,
/// use thread-safe interior mutability in your implementation:
///
/// ```rust
/// use std::sync::atomic::{AtomicI64, Ordering};
/// use bobbin_runtime::{HostState, Value};
///
/// struct GameState {
///     player_health: AtomicI64,
///     gold: AtomicI64,
/// }
///
/// impl HostState for GameState {
///     fn lookup(&self, name: &str) -> Option<Value> {
///         match name {
///             "player_health" => Some(Value::Number(self.player_health.load(Ordering::Relaxed) as f64)),
///             "gold" => Some(Value::Number(self.gold.load(Ordering::Relaxed) as f64)),
///             _ => None,
///         }
///     }
/// }
///
/// // Game can update values anytime:
/// // game_state.player_health.store(50, Ordering::Relaxed);
/// ```
pub trait HostState: Send + Sync {
    /// Look up a host variable by name.
    ///
    /// Returns `Some(value)` if the variable exists, `None` otherwise.
    /// A `None` return will cause `RuntimeError::MissingExternVariable` at runtime.
    fn lookup(&self, name: &str) -> Option<Value>;
}
