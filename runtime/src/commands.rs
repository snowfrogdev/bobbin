//! Command handling for Bobbin scripts.
//!
//! Commands are fire-and-forget calls that trigger game-side effects
//! (e.g., giving items, playing sounds). They do not return values.

use crate::chunk::Value;
use std::fmt;

/// Handler for command invocations from Bobbin scripts.
///
/// Commands are fire-and-forget calls that trigger game-side effects
/// (e.g., giving items, playing sounds). They do not return values.
///
/// # Thread Safety
/// Implementations must be `Send + Sync` as the runtime may be shared
/// across threads in multi-threaded game engines.
///
/// # Example
/// ```ignore
/// use bobbin_runtime::{CommandHandler, CommandError, Value};
///
/// struct GameCommandHandler {
///     // game state references...
/// }
///
/// impl CommandHandler for GameCommandHandler {
///     fn invoke(&self, name: &str, args: &[Value]) -> Result<(), CommandError> {
///         match name {
///             "give_gold" => {
///                 let amount = match &args[0] {
///                     Value::Number(n) => *n as i32,
///                     _ => return Err(CommandError::InvalidArgument {
///                         index: 0,
///                         message: "expected number".to_string(),
///                     }),
///                 };
///                 // self.player.add_gold(amount);
///                 Ok(())
///             }
///             _ => Err(CommandError::UnknownCommand(name.to_string()))
///         }
///     }
///
///     fn is_registered(&self, name: &str) -> bool {
///         matches!(name, "give_gold" | "play_sound" | "give_item")
///     }
///
///     fn arity(&self, name: &str) -> Option<usize> {
///         match name {
///             "give_gold" => Some(1),
///             "play_sound" => Some(1),
///             "give_item" => Some(2),
///             _ => None,
///         }
///     }
/// }
/// ```
pub trait CommandHandler: Send + Sync {
    /// Invokes a command with the given arguments.
    ///
    /// # Arguments
    /// * `name` - The command name as declared in the script (e.g., "give_gold")
    /// * `args` - The evaluated argument values, in declaration order
    ///
    /// # Returns
    /// * `Ok(())` - Command executed successfully
    /// * `Err(CommandError)` - Command failed (execution may continue based on error handling)
    fn invoke(&self, name: &str, args: &[Value]) -> Result<(), CommandError>;

    /// Returns whether a command with the given name is registered.
    ///
    /// Called during semantic analysis to validate command declarations.
    fn is_registered(&self, name: &str) -> bool;

    /// Returns the expected number of arguments for a command.
    ///
    /// Called during semantic analysis to validate argument counts.
    /// Returns `None` if the command is not registered.
    fn arity(&self, name: &str) -> Option<usize>;
}

/// Error type for command invocation failures.
#[derive(Debug, Clone, PartialEq)]
pub enum CommandError {
    /// Command name was not recognized by the handler.
    UnknownCommand(String),
    /// Wrong number of arguments provided.
    ArityMismatch { expected: usize, got: usize },
    /// Argument type was invalid.
    InvalidArgument { index: usize, message: String },
    /// Command execution failed for a custom reason.
    ExecutionFailed(String),
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownCommand(name) => write!(f, "unknown command: {}", name),
            Self::ArityMismatch { expected, got } => {
                write!(f, "expected {} arguments, got {}", expected, got)
            }
            Self::InvalidArgument { index, message } => {
                write!(f, "invalid argument {}: {}", index, message)
            }
            Self::ExecutionFailed(msg) => write!(f, "command failed: {}", msg),
        }
    }
}

impl std::error::Error for CommandError {}

/// No-op handler for scripts without commands.
///
/// All commands are considered unregistered, and invocations are no-ops.
/// This is the default handler used when no commands are configured.
pub struct NoopCommandHandler;

impl CommandHandler for NoopCommandHandler {
    fn invoke(&self, _name: &str, _args: &[Value]) -> Result<(), CommandError> {
        Ok(())
    }

    fn is_registered(&self, _name: &str) -> bool {
        false
    }

    fn arity(&self, _name: &str) -> Option<usize> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_handler_returns_false_for_is_registered() {
        let handler = NoopCommandHandler;
        assert!(!handler.is_registered("give_gold"));
        assert!(!handler.is_registered("play_sound"));
    }

    #[test]
    fn noop_handler_returns_none_for_arity() {
        let handler = NoopCommandHandler;
        assert_eq!(handler.arity("give_gold"), None);
        assert_eq!(handler.arity("play_sound"), None);
    }

    #[test]
    fn noop_handler_invoke_succeeds() {
        let handler = NoopCommandHandler;
        assert!(handler.invoke("give_gold", &[Value::Number(100.0)]).is_ok());
    }

    #[test]
    fn command_error_display() {
        assert_eq!(
            CommandError::UnknownCommand("foo".to_string()).to_string(),
            "unknown command: foo"
        );
        assert_eq!(
            CommandError::ArityMismatch {
                expected: 2,
                got: 1
            }
            .to_string(),
            "expected 2 arguments, got 1"
        );
        assert_eq!(
            CommandError::InvalidArgument {
                index: 0,
                message: "expected number".to_string()
            }
            .to_string(),
            "invalid argument 0: expected number"
        );
        assert_eq!(
            CommandError::ExecutionFailed("database error".to_string()).to_string(),
            "command failed: database error"
        );
    }
}
