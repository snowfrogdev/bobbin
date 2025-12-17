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

**Decision**: Dynamic typing with runtime coercion at host boundaries.

Bobbin supports these internal types:
- `bool` - true/false
- `int` - integers
- `float` - floating-point numbers
- `string` - text
- `table` - key-value pairs (syntax TBD)

Type checking happens at runtime. When values cross the boundary between Bobbin and the host engine (Godot, Unity, etc.), Bobbin performs type coercion as needed.

**Rationale**:
- Dynamic typing reduces friction for narrative designers
- Mandatory type declarations add verbosity without proportional benefit for a dialogue DSL
- Cross-engine compatibility is simpler when Bobbin handles type mapping internally

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
2. Its `save` declarations become available in all other dialogue files
3. No explicit import syntax required

**Rationale**: Enables shared state across files without exposing module system complexity. The infrastructure supports a future explicit `import` statement.

### Name Collision Handling

**Decision**: Dialogue global with same name as game variable = compile error.

If the game provides a variable called `gold`, and a dialogue file declares:

```
save gold = 100
```

This produces a compile error: "Cannot declare 'gold': name conflicts with game variable."

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

## Implementation Notes

### Scanner Token Types

When implementing, the scanner should recognize these line prefixes:

| Prefix | Token | Example |
|--------|-------|---------|
| `save ` | SAVE | `save x = 0` |
| `temp ` | TEMP | `temp y = 0` |
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

### VariableStorage Interface

```rust
pub trait VariableStorage {
    fn get(&self, name: &str) -> Option<Value>;
    fn set(&mut self, name: &str, value: Value);
    fn contains(&self, name: &str) -> bool;
    fn keys(&self) -> Vec<String>;
}
```
