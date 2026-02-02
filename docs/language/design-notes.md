# Bobbin Language Design Notes

This document captures design decisions that don't warrant full ADRs, plus tracks decisions that are still pending.

## Decided

### Visit Tracking

**Decision**: Automatic visit tracking for choice sets.

Bobbin automatically tracks how many times each choice set has been visited. This count is stored as part of the dialogue state and persisted alongside dialogue globals.

Writers can query visit counts to vary dialogue:

```bobbin
# Visit tracking syntax TBD, but conceptually:
if tavern_choice.visits > 0
    Welcome back to the tavern!
else
    You enter a dimly lit tavern.
```

**Rationale**: Automatic tracking eliminates boilerplate. In a game with 500 choice sets, manual tracking would require 500 boolean variables. Ink pioneered this approach and it significantly improves writer productivity.

### Type System

**Decision**: Hybrid static/dynamic typing based on variable category (see ADR-0004).

Bobbin supports these internal types:

- `bool` - true/false
- `number` - floating-point numbers (f64), displayed as integers when whole
- `string` - text
- `table` - key-value pairs (planned, not yet implemented)

The planned type checking varies by variable category:

| Category | Keyword | Typing | Checking |
|----------|---------|--------|----------|
| Temporaries | `temp` | Static | Compile-time |
| Dialogue globals | `save` | Static | Compile-time + runtime verification |
| Host variables | `extern` | Dynamic | Runtime only |

> **Current implementation**: All variables are dynamically typed. Static type checking for `temp` and `save` is planned but not yet implemented.

### Host Variable Declaration (`extern`)

**Decision**: Use `extern` keyword to declare host-provided variables.

Bobbin scripts must explicitly declare which variables they expect from the host:

```bobbin
extern player_health
extern gold
extern player_name

Welcome, {player_name}! You have {player_health} HP and {gold} gold.
```

**Semantics:**

- Declares a variable exists but is provided by host, not dialogue
- No initial value (host owns the value)
- Read-only: attempting `set player_health = 100` is a semantic error
- Must be declared before use
- Duplicate declarations in same file are errors; across files are OK (idempotent)
- If host doesn't provide the variable at runtime, `RuntimeError::MissingExternVariable`

**Rationale:**

- Self-documenting: scripts explicitly list their host dependencies
- Compile-time validation: typos caught early (`{playr_health}` → error if not declared)
- Tooling support: IDEs can autocomplete declared variables
- Prelude-compatible: common externs can go in `globals.bobbin`

See ADR-0004 for the two-interface architecture.

### Global Initialization Semantics

**Decision**: "Default" semantics - don't overwrite if exists.

When Bobbin encounters:

```
save merchant_relationship = 0
```

It checks if `merchant_relationship` already exists in storage:
- If **not present**: create with value `0`
- If **present**: leave existing value alone

**Rationale**: This prevents save games from resetting progress. When a player loads a save, dialogue files are reloaded, but their `save` declarations don't overwrite persisted values.

### Cross-File Globals (Prelude System)

**Decision**: `globals.bobbin` is automatically loaded first if present.

The prelude system:
1. If `globals.bobbin` exists in the project, load it first
2. Its `save` and `extern` declarations become available in all other dialogue files
3. No explicit import syntax required

**Rationale**: Enables shared state across files without exposing module system complexity. The infrastructure supports a future explicit `import` statement.

### Name Collision Handling

**Decision**: Shadowing between variable categories is a semantic error.

If a dialogue file declares conflicting variables:

```bobbin
extern gold
save gold = 100    # Semantic error: shadows extern
```

Or:

```bobbin
save gold = 100
extern gold        # Semantic error: shadows save
```

This produces a semantic error caught by the resolver. Duplicate `extern` declarations in the same file are errors; across files they are allowed (idempotent).

**Rationale**: Silent shadowing would cause confusion. Explicit errors make the conflict visible.

### Interpolation

**Decision**: Curly brace delimiters with variable-only content (Phase 1).

Bobbin uses `{variable_name}` to embed variable values in dialogue text:

```
save player_name = "Hero"
save gold = 100

Welcome, {player_name}! You have {gold} gold pieces.
```

**Escape mechanism**: Use `{{` for a literal `{` character, `}}` for literal `}`:

```
To show braces: {{like this}}
```

**Rationale**:

- Familiar syntax (matches Ink, Python f-strings, C#, Yarn Spinner)
- Clean in prose - minimal visual noise
- `{{` escape is standard and intuitive

**Alternatives considered**:

- `[var]` (Ren'Py style) - less familiar; conflicts with future array access
- `${var}` (JavaScript style) - more verbose; `$` might conflict with variable prefixes
- `$var` (naked sigil) - ambiguous boundaries (`$goldfish`?)

**Current scope**: Variable names, comparison expressions, logical expressions, and arithmetic expressions are all supported inside `{...}`. Function calls are TBD for a future phase.

**Comparison expressions** (implemented):

- Equality: `{x == y}`, `{x != y}` - works on any same-typed values
- Ordering: `{x < y}`, `{x <= y}`, `{x > y}`, `{x >= y}` - numbers only

**Arithmetic expressions** (implemented):

- Binary: `{x + y}`, `{x - y}`, `{x * y}`, `{x / y}`, `{x % y}` - numbers only
- Unary: `{-x}` for negation
- Parentheses: `{(a + b) * c}` for grouping
- Division/modulo by zero produces a runtime error

**Logical expressions** (implemented):

- `and`, `or`, `not` — word-based operators for clarity in prose-heavy scripts
- Short-circuit evaluation: `false and x` doesn't evaluate `x`
- Can combine with comparisons: `{health > 0 and gold >= price}`

**Operator precedence** (lowest to highest):

1. `or` (logical or)
2. `and` (logical and)
3. `==`, `!=` (equality)
4. `<`, `<=`, `>`, `>=` (comparison)
5. `+`, `-` (addition, subtraction)
6. `*`, `/`, `%` (multiplication, division, modulo)
7. `not`, Unary `-` (logical not, negation)
8. `()` (parentheses)

### Conditional Syntax

**Decision**: Python-style indentation-based blocks with `if`/`elseif`/`else`.

```bobbin
if gold >= 100
    You can afford the premium item!
elseif gold >= 50
    You can afford the standard item.
else
    You need more gold.
```

**Syntax details:**

- `if <expression>` — no parentheses required, no trailing colon
- `elseif <expression>` — for additional conditions (not `elif`)
- `else` — optional final branch
- Indentation defines block scope (4 spaces recommended)
- Conditions must evaluate to boolean

**Interaction with choices:**

Conditionals can contain choice sets, and choices can contain conditionals:

```bobbin
if show_inventory
    - Check items
        You have {item_count} items.
    - Leave
        Goodbye.
else
    The inventory is locked.
```

**Rationale:**

- Indentation-based blocks match dialogue's natural structure
- `elseif` is more readable in prose than `elif`
- No colons/parentheses reduces visual noise

### Commands (Dialogue-to-Game Effects)

**Decision**: Function-style syntax with required declarations.

Commands allow Bobbin scripts to trigger game effects like giving items, playing sounds, or completing quests:

```bobbin
extern give_gold(amount)
extern play_sound(name)
extern give_item(name, count)

give_gold(100)
play_sound("coin")
give_item("sword", 1)
```

**Syntax:**

- **Declaration**: `extern command_name(param1, param2, ...)` — declares a command the host provides
- **Invocation**: `command_name(expr1, expr2, ...)` — calls the command with arguments

**Semantics:**

- Fire-and-forget: commands do not return values
- Arguments are expressions: literals, variables, arithmetic, etc.
- Argument count is validated at compile time against the declaration
- Commands are provided by the game engine at runtime creation
- Command names must not conflict with variable names

**API:**

Commands are registered via the `CommandHandler` trait:

```rust
pub trait CommandHandler: Send + Sync {
    fn invoke(&self, name: &str, args: &[Value]) -> Result<(), CommandError>;
    fn is_registered(&self, name: &str) -> bool;
    fn arity(&self, name: &str) -> Option<usize>;
}
```

**GDScript:**

```gdscript
var runtime = Bobbin.create("res://dialogue/merchant.bobbin", {}, {}, {
    "give_gold": func(args): player.gold += int(args[0]),
    "play_sound": func(args): AudioManager.play(args[0]),
})
```

**Rationale:**

- Function-style syntax is familiar and supports multiple arguments
- Required declarations enable compile-time validation of typos and arity
- Fire-and-forget semantics keep the model simple; use `extern` variables for queries
- `extern` keyword reuse maintains consistency with extern variables

**Alternatives considered:**

- Keyword-style (`command give_gold 100`) — less extensible for multiple args
- Signal-based — more complex for simple effects
- Return values — adds complexity; can use extern variables for queries instead

See ADR-0005 for the full design rationale.

## To Be Decided

The following design decisions need to be made before implementation:

### Expression Syntax

**Implemented**:

- Comparison operators: `==`, `!=`, `<`, `<=`, `>`, `>=`
- Arithmetic operators: `+`, `-`, `*`, `/`, `%`
- Logical operators: `and`, `or`, `not`
- Unary negation: `-x`
- Parentheses for grouping: `(a + b) * c`
- Operator precedence (standard mathematical order)

**Remaining questions**:

- String concatenation operator?

### Table Syntax

**Questions**:
- Literal syntax: `{}`, `table()`, something else?
- Access syntax: `table["key"]`, `table.key`, both?
- Safe access method: `.get(key, default)` or `table["key"] or default`?
- Membership test: `"key" in table` or `table.has("key")`?

### Interpolation Expressions

**Implemented**:

- Comparison operators: `{x == y}`, `{x != y}`, `{x < y}`, `{x <= y}`, `{x > y}`, `{x >= y}`
- Arithmetic operators: `{x + y}`, `{x - y}`, `{x * y}`, `{x / y}`, `{x % y}`
- Logical operators: `{x and y}`, `{x or y}`, `{not x}`
- Unary negation: `{-x}`
- Parentheses for grouping: `{(a + b) * c}`

**Remaining questions**:

- Function calls: `{get_title(npc)}`?
- Inline conditionals: `{if gold > 0 then "some" else "no"}`?

Note: Basic interpolation syntax (`{var}` and `{{` escape), comparison expressions, logical expressions, and arithmetic expressions are decided - see "Decided" section above.

### Compound Assignment

**Questions**:
- Support `+=`, `-=`, `*=`, `/=`?
- String concatenation: `+=` for strings?

### Module System

**Questions**:
- Explicit import syntax: `import foo from "file.bobbin"`?
- Export syntax needed?
- Circular dependency handling?

## Implementation Notes

### Scanner Token Types

When implementing, the scanner should recognize these line prefixes:

| Prefix | Token | Example |
|--------|-------|---------|
| `save ` | SAVE | `save x = 0` |
| `temp ` | TEMP | `temp y = 0` |
| `extern ` | EXTERN | `extern player_health` |
| `extern ` + `(` | EXTERN_COMMAND | `extern give_gold(amount)` |
| `set ` | SET | `set x = 1` |
| `if ` | IF | `if condition` |
| `elseif ` | ELSEIF | `elseif other` |
| `else` | ELSE | `else` |
| `- ` | CHOICE | `- Option text` |
| `identifier(` | COMMAND_CALL | `give_gold(100)` |
| (other) | LINE | `Dialogue text` |

### Value Type Enum

```rust
pub enum Value {
    Bool(bool),
    Number(f64),
    String(String),
    // Table(HashMap<String, Value>),  // Planned, not yet implemented
}
```

### VariableStorage Interface (Dialogue Globals)

```rust
pub trait VariableStorage {
    /// Get the current value of a dialogue global
    fn get(&self, name: &str) -> Option<Value>;

    /// Set a dialogue global to a new value
    fn set(&mut self, name: &str, value: Value);

    /// Initialize only if absent (for `save` declarations)
    fn initialize_if_absent(&mut self, name: &str, default: Value);

    /// Check if a variable exists
    fn contains(&self, name: &str) -> bool;
}
```

### HostState Interface (Host Variables)

```rust
pub trait HostState {
    /// Look up a host variable (read-only from Bobbin's perspective)
    fn lookup(&self, name: &str) -> Option<Value>;
}
```

See ADR-0004 for the rationale behind two separate interfaces.
