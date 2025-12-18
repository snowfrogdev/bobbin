---
status: Accepted
date: 2025-01-17
deciders: Phil
---

# Variable Storage Architecture and Type System

## Context and Problem Statement

ADR-0002 established a three-tier variable model (host variables, dialogue globals, temporaries) and specified that the host provides a `VariableStorage` interface. However, it left open important questions:

1. **Read/write patterns**: Should dialogue be able to write to host variables, or only read them?
2. **Interface design**: Is one interface sufficient, or should different variable categories use different interfaces?
3. **Type system**: How should types be enforced across the storage boundary?

These questions arise because different use cases have different requirements:

| Use Case | Example | Read/Write Pattern |
|----------|---------|-------------------|
| Dialogue-declared, dialogue-read | `save visited_tavern = false` then `{visited_tavern}` | Bobbin owns entirely |
| Dialogue-declared, host-read | Quest progress the host checks | Bobbin writes, host reads |
| Host-declared, dialogue-read | `extern player_health` then `{player_health}` | Host owns, Bobbin reads |
| Host-declared, dialogue-write | `set gold = gold + 100` | Semantic error - extern is read-only |

## Decision Drivers

- **Clear ownership**: Each variable should have one authoritative owner
- **Type safety**: Catch errors early where possible, but handle cross-boundary realities
- **Host control**: Hosts must not lose control of their own state
- **Writer productivity**: Dialogue authors shouldn't need programmer help for dialogue-only state
- **Cross-engine compatibility**: Works with Godot, Unity, and other hosts

## Considered Options

### Interface Design

1. **Single interface**: One `VariableStorage` for all persistent state
2. **Two interfaces**: Separate `VariableStorage` (dialogue-owned) and `HostState` (host-owned, read-only)
3. **Three interfaces**: Add a third for commands/events

### Type System

4. **Fully dynamic**: All variables are dynamically typed
5. **Fully static**: All variables have compile-time types
6. **Hybrid static/dynamic**: Different categories have different typing rules

## Decision Outcome

Chosen options:
- **Two interfaces** (Option 2) for clear ownership semantics
- **Hybrid static/dynamic typing** (Option 6) for practical type safety

### Two-Interface Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Host Application                     │
│  ┌────────────────────────┐  ┌────────────────────────┐    │
│  │   VariableStorage      │  │      HostState         │    │
│  │   (dialogue globals)   │  │   (host variables)     │    │
│  │   - read/write         │  │   - read-only          │    │
│  │   - host serializes    │  │   - host owns entirely │    │
│  └────────────────────────┘  └────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
                    │                        │
                    │ get/set/initialize     │ lookup
                    ▼                        ▼
┌─────────────────────────────────────────────────────────────┐
│                    Bobbin Runtime                           │
│  ┌────────────────────┐  ┌────────────────────────┐        │
│  │  Dialogue Globals  │  │   Temporary Variables  │        │
│  │  (via storage)     │  │   (stack only)         │        │
│  └────────────────────┘  └────────────────────────┘        │
└─────────────────────────────────────────────────────────────┘
```

#### VariableStorage Interface (Dialogue Globals)

```rust
pub trait VariableStorage {
    /// Get the current value of a dialogue global
    fn get(&self, name: &str) -> Option<Value>;

    /// Set a dialogue global to a new value
    fn set(&mut self, name: &str, value: Value);

    /// Initialize a variable only if it doesn't exist (for `save` declarations)
    fn initialize_if_absent(&mut self, name: &str, default: Value);

    /// Check if a variable exists
    fn contains(&self, name: &str) -> bool;
}
```

Used for:
- `save` variable declarations
- `set` modifications to dialogue globals
- Reading dialogue globals via interpolation

#### HostState Interface (Host Variables)

```rust
pub trait HostState {
    /// Look up a host variable (read-only from Bobbin's perspective)
    fn lookup(&self, name: &str) -> Option<Value>;
}
```

Used for:
- Reading host variables like `{player_health}`, `{gold}` (declared via `extern`)
- Condition checks involving host state

Host variables are read-only from Bobbin's perspective. Attempting to use `set` on an extern variable is a semantic error caught at compile time by the resolver.

### Type System

| Category | Keyword | Typing | Checking | Rationale |
|----------|---------|--------|----------|-----------|
| Temporaries | `temp` | Static | Compile-time | Stack-only, never crosses boundaries |
| Dialogue globals | `save` | Static | Compile + runtime | Declared type is expected; storage verified |
| Host variables | `extern` | Dynamic | Runtime | Type unknown until lookup |

#### Temporary Variables (`temp`)

Fully static typing. The compiler tracks types and catches mismatches at compile time.

```
temp counter = 0        # Type: int
temp name = "Hero"      # Type: string
set counter = "oops"    # Compile error: expected int, got string
```

#### Dialogue Globals (`save`)

Static typing with runtime verification. The declared type establishes the expected type, which is checked at compile time. When values cross the storage boundary, runtime verification ensures the stored value matches the expected type.

```
save reputation = 0     # Declared type: int

# Compile-time: compiler knows reputation is int
set reputation = 10     # OK: int assigned to int
set reputation = "bad"  # Compile error: expected int

# Runtime: storage returns value, runtime verifies type
# If storage contains corrupted data (e.g., string), runtime error occurs
```

The trust boundary is at the `VariableStorage` interface. If the host's storage implementation returns a value of the wrong type, that's a runtime error - but this should be rare in practice since Bobbin controls what gets written.

#### Host Variables (`extern`)

Fully dynamic typing. The host can provide any type, and Bobbin discovers it at runtime.

```bobbin
extern player_health

# player_health could be int, float, or even string
# Bobbin discovers the type when it calls HostState::lookup()
You have {player_health} HP.
```

The `extern` keyword declares that a variable is provided by the host application. It must be declared before use. Type mismatches in expressions involving host variables are runtime errors.

If the host doesn't provide a declared extern variable at runtime, `RuntimeError::MissingExternVariable` is raised.

### Dialogue-to-Host Effects

For cases where dialogue should affect host state (giving items, triggering events), the recommended pattern is **commands/events** rather than direct writes:

```bobbin
# Instead of:
set gold = gold + 100        # Would bypass host's economy logic (and is a semantic error for extern)

# Use commands (syntax TBD):
give_gold(100)               # Host implements the command
trigger_event("quest_complete")
```

Commands ensure the host maintains control of its state while allowing dialogue to request effects. The exact syntax is deferred to a future ADR.

### Consequences

- Good, because ownership is clear: dialogue owns `save` variables, host owns `extern` variables
- Good, because the host cannot accidentally have its state corrupted by dialogue (extern is read-only)
- Good, because static typing catches most errors at compile time
- Good, because writers get full autonomy over dialogue state
- Good, because the runtime can give clear error messages for type mismatches
- Good, because `extern` declarations make host dependencies explicit and self-documenting
- Bad, because two interfaces require slightly more integration work than one
- Bad, because runtime type verification adds overhead (minimal in practice)
- Neutral, because commands for host effects require a future design decision

## Pros and Cons of the Options

### Single Interface (Option 1)

- Good, because simpler integration (one interface)
- Bad, because conflates ownership (who controls what?)
- Bad, because dialogue could write to host variables, bypassing host logic
- Bad, because type safety is harder without clear ownership

### Fully Dynamic Typing (Option 4)

- Good, because simpler implementation (no type tracking)
- Good, because maximum flexibility
- Bad, because common errors (type mismatches) only surface at runtime
- Bad, because no IDE support for type hints
- Bad, because errors may only appear in specific playtest scenarios

### Fully Static Typing (Option 5)

- Good, because all errors caught at compile time
- Good, because enables rich IDE support
- Bad, because host variables have unknown types until runtime
- Bad, because requires type annotations that add verbosity
- Bad, because cross-language type mapping (GDScript, C#, Rust) is complex

## More Information

### Resolution Order

When looking up a variable, the resolver checks in order:

1. **Local scope**: `temp` variables in current and enclosing scopes
2. **Dialogue globals**: `save` variables from current file and prelude
3. **Host state**: Variables declared with `extern` (provided by `HostState` at runtime)

This order ensures local variables shadow globals. Shadowing between categories (`temp`/`save`/`extern`) is a semantic error caught by the resolver.

### Default Initialization Semantics

The `save` declaration uses "initialize if absent" semantics:

```
save merchant_relationship = 0
```

This means:
- If not in storage: create with value `0` and type `int`
- If already in storage: verify type is `int`, keep existing value

This prevents save games from resetting progress when dialogue files reload.

### Related Decisions

- ADR-0002: Variable and State Management (establishes the three-tier model)
- ADR-0003: Variable Modification Syntax (covers `set` keyword)
- Future ADR: Command syntax for dialogue-to-host effects
