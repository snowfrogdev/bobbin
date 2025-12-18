# Bobbin Language Design Notes

This document captures design decisions that don't warrant full ADRs, plus tracks decisions that are still pending.

## Decided

### Visit Tracking

**Decision**: Automatic visit tracking for choice sets.

Bobbin automatically tracks how many times each choice set has been visited. This count is stored as part of the dialogue state and persisted alongside dialogue globals.

Writers can query visit counts to vary dialogue:

```
# Syntax TBD, but conceptually:
if tavern_choice.visits > 0:
    Welcome back to the tavern!
else:
    You enter a dimly lit tavern.
```

**Rationale**: Automatic tracking eliminates boilerplate. In a game with 500 choice sets, manual tracking would require 500 boolean variables. Ink pioneered this approach and it significantly improves writer productivity.

### Type System

**Decision**: Hybrid static/dynamic typing based on variable category.

Bobbin supports these internal types:
- `bool` - true/false
- `int` - integers
- `float` - floating-point numbers
- `string` - text
- `table` - key-value pairs (syntax TBD)

Type checking varies by variable category:

| Category | Keyword | Typing | Checking |
|----------|---------|--------|----------|
| Temporaries | `temp` | Static | Compile-time |
| Dialogue globals | `save` | Static | Compile-time + runtime verification |
| Host variables | `extern` | Dynamic | Runtime only |

**Temporary variables** (`temp`) are fully statically typed. The compiler infers the type from the initial value and catches type mismatches at compile time.

**Dialogue globals** (`save`) have static typing with runtime verification. The declared type is checked at compile time, and the runtime verifies that values retrieved from storage match the expected type. If the storage returns a value of the wrong type, that's a runtime error.

**Host variables** (`extern`) are dynamically typed. Since the host provides these values at runtime, Bobbin cannot know their types until lookup. Type mismatches in expressions are runtime errors.

**Rationale**:
- Static typing for `temp` and `save` catches most errors at compile time
- Runtime verification at the storage boundary handles the reality that storage is external
- Dynamic typing for host variables is necessary since types are unknown until runtime
- This balance maximizes error detection while remaining practical

See ADR-0004 for the full architecture.

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

**Phase 1 scope**: Only variable names are allowed inside `{...}`. Arithmetic expressions and function calls are TBD for a future phase.

## To Be Decided

The following design decisions need to be made before implementation:

### Expression Syntax

**Questions**:
- Operator symbols: `and`/`or`/`not` vs `&&`/`||`/`!`?
- Operator precedence?
- Parentheses for grouping?
- String concatenation operator?

### Conditional Syntax

**Questions**:
- `if`/`else` or `if`/`elif`/`else`?
- Indentation-based blocks (Python-style)?
- Condition syntax: `if condition:` or `if (condition)`?
- How do conditionals interact with choices?

### Table Syntax

**Questions**:
- Literal syntax: `{}`, `table()`, something else?
- Access syntax: `table["key"]`, `table.key`, both?
- Safe access method: `.get(key, default)` or `table["key"] or default`?
- Membership test: `"key" in table` or `table.has("key")`?

### Interpolation Expressions

**Questions**:

- What expressions beyond variable names should be allowed inside `{...}`?
- Arithmetic: `{gold * 2}`?
- Function calls: `{get_title(npc)}`?
- Inline conditionals: `{if gold > 0 then "some" else "no"}`?

Note: Basic interpolation syntax (`{var}` and `{{` escape) is decided - see "Decided" section above.

### Compound Assignment

**Questions**:
- Support `+=`, `-=`, `*=`, `/=`?
- String concatenation: `+=` for strings?

### Module System

**Questions**:
- Explicit import syntax: `import foo from "file.bobbin"`?
- Export syntax needed?
- Circular dependency handling?

### Dialogue-to-Game Effects (Commands)

**Context**: Dialogue may need to trigger game effects (give items, complete quests, play sounds). Direct writes to game variables would bypass game logic, so a command/event system is preferred.

**Questions**:
- Command syntax: function-style `give_gold(100)` or keyword-style `command give_gold 100`?
- How are commands registered by the game?
- Return values from commands?
- Error handling for unknown commands?

See ADR-0004 for the architectural rationale.

## Implementation Notes

### Scanner Token Types

When implementing, the scanner should recognize these line prefixes:

| Prefix | Token | Example |
|--------|-------|---------|
| `save ` | SAVE | `save x = 0` |
| `temp ` | TEMP | `temp y = 0` |
| `extern ` | EXTERN | `extern player_health` |
| `set ` | SET | `set x = 1` |
| `- ` | CHOICE | `- Option text` |
| (other) | LINE | `Dialogue text` |

### Value Type Enum

```rust
pub enum Value {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Table(HashMap<String, Value>),
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
