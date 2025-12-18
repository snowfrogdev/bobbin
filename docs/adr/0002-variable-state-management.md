---
status: Accepted
date: 2025-01-16
deciders: Phil
---

# Variable and State Management Architecture

## Context and Problem Statement

Bobbin is a dialogue DSL that needs variable and state tracking to support branching narratives. Writers need to:

- Track conversation history (has this NPC been met before?)
- Store relationship scores and quest progress
- Access game state (player health, inventory)
- Have changes persist across save/load cycles

How should Bobbin manage state across dialogue execution, game integration, and persistence?

## Decision Drivers

- **Writer autonomy**: Writers should be able to create persistent dialogue variables without requiring a programmer to modify game code
- **Game integration**: Dialogue must access game state (health, gold, inventory) provided by the host engine
- **Save/load support**: Dialogue state must survive save/load cycles
- **Hot-reload**: Writers must iterate on dialogue while the game runs, preserving variable state
- **Cross-engine compatibility**: Bobbin targets multiple hosts (Godot, Unity, etc.) with different type systems
- **Simplicity**: Minimize cognitive overhead for narrative designers

## Considered Options

### Variable Tier Models

1. **Two-tier: Game variables + temporary locals**
   - Game owns all persistent state
   - Dialogue can only have temporary (non-persisted) variables
   - Writers must ask programmers to add any persistent variable

2. **Three-tier: Game variables + dialogue globals + temporary locals**
   - Game provides some variables (health, gold)
   - Dialogue can declare its own persistent globals (`save`)
   - Dialogue can declare temporary locals (`temp`)

### Type System Approaches

3. **Static typing**: Variables declared with explicit types, checked at compile time
4. **Dynamic typing**: Variables can hold any value, types checked at runtime

### Persistence Models

5. **Bobbin-owned persistence**: Bobbin serializes/deserializes its own state
6. **Game-owned persistence**: Game provides opaque storage, controls serialization

## Decision Outcome

Chosen option: **Three-tier variable model with dynamic typing and host-owned persistence**

### The Three Tiers

| Tier | Keyword | Declared By | Persisted By | Example |
|------|---------|-------------|--------------|---------|
| Host variables | `extern` | Dialogue file | Host | `extern player_health` |
| Dialogue globals | `save` | Dialogue file | Host | `save merchant_relationship = 0` |
| Temporaries | `temp` | Dialogue file | Not persisted | `temp loop_counter = 0` |

### Persistence Architecture

The host provides two interfaces. Bobbin treats them as opaque:

```
┌─────────────────────────────────────────────────────────┐
│                     Host Application                    │
│  ┌────────────────────────┐  ┌──────────────────────┐  │
│  │   VariableStorage      │  │     HostState        │  │
│  │   (dialogue globals)   │  │  (host variables)    │  │
│  │   - read/write         │  │  - read-only         │  │
│  │   - host serializes    │  │  - host owns         │  │
│  └────────────────────────┘  └──────────────────────┘  │
└─────────────────────────────────────────────────────────┘
                    │                      │
                    │ get/set/init         │ lookup
                    ▼                      ▼
┌─────────────────────────────────────────────────────────┐
│                    Bobbin Runtime                       │
│  ┌────────────────────┐  ┌────────────────────────┐    │
│  │  Dialogue Globals  │  │   Temporary Variables  │    │
│  │  (via storage)     │  │   (runtime only)       │    │
│  └────────────────────┘  └────────────────────────┘    │
└─────────────────────────────────────────────────────────┘
```

See ADR-0004 for details on the two-interface architecture.

### Prelude System for Cross-File Globals

A `globals.bobbin` file (if present) is automatically loaded before other dialogue files. Its `save` and `extern` declarations become available in all files:

```bobbin
# globals.bobbin
extern player_health
extern gold
save merchant_relationship = 0
save quest_main_progress = "not_started"

# shop.bobbin (can use these without re-declaring)
You have {gold} gold and {player_health} HP.
set merchant_relationship = 10
```

This provides a foundation for a future module/import system without exposing that complexity now.

### Global Initialization Semantics

The `save` declaration uses "default" semantics:

```
save merchant_relationship = 0
```

Means: "If `merchant_relationship` doesn't exist in storage, create it with value `0`. If it already exists, leave it alone."

This prevents save games from resetting progress when dialogue files are loaded.

### Consequences

- Good, because writers can create persistent variables (`save`) without programmer involvement
- Good, because game maintains control of its save system
- Good, because the VariableStorage interface works across host languages (Godot, Unity, etc.)
- Good, because dynamic typing avoids complex type mapping between host languages
- Good, because hot-reload works naturally: swap bytecode, preserve storage contents
- Good, because the prelude system enables shared state without import syntax
- Bad, because dynamic typing means some errors surface at runtime rather than compile time
- Bad, because the game must implement the VariableStorage interface (small integration cost)
- Neutral, because three tiers add some complexity vs. two, but the writer autonomy benefit outweighs this

## Pros and Cons of the Options

### Two-Tier Model (Game + Temporary Only)

- Good, because it's simpler (only two categories)
- Good, because game has full control of all persistent state
- Bad, because writers must involve programmers for any persistent dialogue state
- Bad, because common patterns (relationship tracking, visit counting) require code changes

### Static Typing

- Good, because errors are caught at compile time
- Good, because enables better tooling (autocomplete with types)
- Bad, because requires type declarations (more verbose)
- Bad, because type mapping across host languages (GDScript, C#, Rust) is complex
- Bad, because adds friction for narrative designers

### Bobbin-Owned Persistence

- Good, because Bobbin controls serialization format
- Bad, because game loses control of save system
- Bad, because two save systems must be coordinated
- Bad, because game cannot easily inspect dialogue state

## More Information

### Name Collision Handling

Shadowing between variable categories is a semantic error caught by the resolver:

- If a dialogue file declares `save gold = 0` but also has `extern gold`, this is a semantic error
- If a dialogue file declares `temp health = 100` but also has `extern health`, this is a semantic error
- Duplicate `extern` declarations in the same file are errors; across files they are allowed (idempotent)

### Visit Tracking

Bobbin automatically tracks visit counts for choice sets. This is stored as part of the dialogue state and persisted alongside dialogue globals.

### Future Considerations

- **Module/import syntax**: The prelude system provides infrastructure for a future `import` statement
- **Type annotations**: Optional type hints could be added later for tooling benefits without requiring them

### Related Decisions

- ADR-0001: Compiler Architecture (establishes the bytecode VM that executes with this state model)
- ADR-0003: Variable Modification Syntax (covers `set` keyword for modifying variables)
- ADR-0004: Variable Storage Architecture (refines the storage interfaces and type system)
