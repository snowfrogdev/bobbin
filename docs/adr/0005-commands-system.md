# ADR-0005: Commands System

## Status

Accepted

## Context

Bobbin scripts often need to trigger game effects—giving items to the player, playing sounds, completing quests, or updating the game world. The existing variable system handles data exchange:

- **save variables**: Dialogue-owned state that persists across sessions
- **extern variables**: Read-only access to game state

However, neither mechanism supports triggering side effects. Direct writes to game variables would bypass game logic (e.g., inventory capacity checks, achievement triggers, audio systems).

We need a mechanism for dialogue to request game effects while maintaining proper separation of concerns.

## Decision

We introduce **commands**: fire-and-forget function-style calls that trigger game-side effects.

### Syntax

Commands are declared with `extern` and parentheses (distinguishing them from extern variables):

```bobbin
extern give_gold(amount)        # Declares a command with one parameter
extern give_item(name, count)   # Two parameters
extern save_game()              # No parameters
```

Commands are invoked with function-call syntax:

```bobbin
give_gold(100)
give_item("sword", 1)
save_game()
```

### Semantics

1. **Fire-and-forget**: Commands do not return values. For queries, use extern variables.

2. **Required declaration**: Commands must be declared before use. Undeclared command invocations are compile-time errors.

3. **Compile-time arity validation**: The number of arguments at the call site must match the number of parameters in the declaration.

4. **Arguments are expressions**: Command arguments can be literals, variables, or computed expressions:
   ```bobbin
   temp bonus = 10
   give_gold(100 + bonus)  # Expression argument
   ```

5. **No shadowing**: Command names cannot conflict with variable names (and vice versa).

### Runtime API

Commands are provided via the `CommandHandler` trait:

```rust
pub trait CommandHandler: Send + Sync {
    /// Invoke a command with evaluated arguments.
    fn invoke(&self, name: &str, args: &[Value]) -> Result<(), CommandError>;

    /// Check if a command is registered.
    fn is_registered(&self, name: &str) -> bool;

    /// Get the expected argument count.
    fn arity(&self, name: &str) -> Option<usize>;
}
```

Games provide command handlers at runtime creation:

```rust
let handler = Arc::new(MyCommandHandler::new());
let runtime = Runtime::with_commands(script, storage, host, handler)?;
```

### GDScript API

```gdscript
var runtime = Bobbin.create("res://dialogue/merchant.bobbin", {}, {}, {
    "give_gold": func(args): player.gold += int(args[0]),
    "play_sound": func(args): AudioManager.play(args[0]),
    "give_item": func(args): inventory.add(args[0], int(args[1])),
})
```

Or using the config dictionary approach:

```gdscript
var runtime = Bobbin.create_from_config("res://dialogue/merchant.bobbin", {
    "host_state": {"player_name": player.name},
    "commands": {
        "give_gold": func(args): player.gold += int(args[0]),
    }
})
```

## Rationale

### Function-style syntax

We chose function-style `give_gold(100)` over alternatives:

- **Keyword-style** (`command give_gold 100`): Less extensible for multiple arguments, unfamiliar
- **Signal-based** (`emit give_gold(100)`): More complex, implies async behavior we don't need
- **Inline statements** (`!give_gold(100)`): Unusual sigil, less readable

Function syntax is familiar from most programming languages and naturally supports multiple arguments.

### Required declarations

Mandatory `extern give_gold(amount)` declarations provide:

- **Compile-time typo detection**: `give_glod(100)` fails at compile time if not declared
- **Arity validation**: Wrong argument count is caught early
- **Self-documenting scripts**: Reader knows what commands the script depends on
- **Tooling support**: IDE completion, hover info, go-to-definition

### Fire-and-forget semantics

Commands don't return values because:

1. **Simple mental model**: Commands are imperatives, not queries
2. **No blocking**: Game effects (sound playback, animations) may be async
3. **Separation of concerns**: For queries, use extern variables

If a game needs command results, it can update an extern variable that the script reads on the next line:

```bobbin
extern purchase_result
extern try_purchase(item)

try_purchase("sword")
if purchase_result == "success"
    You bought a sword!
else
    You can't afford that.
```

### Reusing `extern` keyword

We reuse `extern` for commands because:

- Both commands and extern variables are "provided by the host"
- Parentheses clearly distinguish `extern player_name` (variable) from `extern give_gold(amount)` (command)
- No new keyword to learn

## Consequences

### Positive

- Clean, familiar syntax for triggering game effects
- Compile-time validation catches common errors
- Clear separation: variables for data, commands for effects
- Easy to implement in game engines (just provide callables)

### Negative

- Commands can't return values (use extern variables instead)
- No built-in async/await (games handle async internally)
- Arity is fixed per command (no variadic commands)

### Neutral

- Games must register all commands at runtime creation
- Command failures propagate as `RuntimeError::CommandFailed`

## Alternatives Considered

### Events/Signals

A pub/sub system where scripts emit events:

```bobbin
emit "gold_changed", 100
```

**Rejected because**: More complex, less readable, requires event name strings, harder to validate.

### Direct writes to game state

Allow `set` on special game-owned variables:

```bobbin
set game.player_gold = game.player_gold + 100
```

**Rejected because**: Bypasses game logic (capacity checks, achievements, etc.), unclear ownership.

### Return values

Commands that return values for use in expressions:

```bobbin
if can_afford("sword")
    buy_item("sword")
```

**Deferred**: Can be added later if needed. Current design uses extern variables for this pattern.
