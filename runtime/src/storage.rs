//! Variable storage interface for dialogue globals.

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
