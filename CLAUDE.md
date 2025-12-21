# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Bobbin is a narrative scripting language for branching dialogue and interactive stories in video games. It consists of:
- **Runtime** (`runtime/`): Core Rust library implementing the language
- **Godot Bindings** (`bindings/godot/`): GDExtension exposing the runtime to Godot 4.3+

## Build Commands

### Runtime Tests
```bash
cargo test -p bobbin-runtime           # Run all tests
cargo test -p bobbin-runtime --test choices  # Run specific test file
```

### Godot Bindings (via Docker)
```bash
docker compose run --rm --build godot windows        # Fast local build (release)
docker compose run --rm --build godot windows debug  # Fast local build (debug)
docker compose run --rm --build godot all            # All platforms × all types

docker compose run --rm --build godot all --ci       # CI builds (optimized, small binaries)
```

Binaries output to `bindings/godot/addons/bobbin/bin/`.

## Architecture

### Compilation Pipeline
```
Source → Scanner → Parser → AST → Resolver → Compiler → Bytecode → VM
```

See `docs/adr/0001-compiler-architecture.md` for the rationale. Key points:
- AST enables tooling (LSP, linter, formatter) without separate parsing
- Bytecode VM supports pause/resume for save games and hot reloading
- Symbol table from semantic analysis enables go-to-definition, autocomplete

### Runtime Structure (`runtime/src/`)
| File | Responsibility |
|------|----------------|
| `scanner.rs` | Tokenizes source, handles indentation (INDENT/DEDENT tokens) |
| `parser.rs` | Recursive descent parser producing AST |
| `resolver.rs` | Semantic analysis: symbol resolution, type checking |
| `compiler.rs` | Tree-walks AST to emit bytecode |
| `vm.rs` | Stack-based bytecode interpreter |
| `storage.rs` | `VariableStorage` and `HostState` traits for game integration |

### Variable System (ADR-0002, ADR-0004)
- **save**: Persistent variables (survive save/load)
- **temp**: Session-scoped variables (cleared on restart)
- **extern**: Read-only variables provided by host game at runtime

Storage uses `Arc<dyn VariableStorage>` with interior mutability (`RwLock`) for thread-safe shared ownership between game and runtime.

### Godot Integration (`bindings/godot/src/lib.rs`)
- `BobbinLanguage`: ScriptLanguageExtension for `.bobbin` file support in editor
- `BobbinScript`: Holds source code for script resources
- `BobbinLoader`/`BobbinSaver`: Resource format loaders for `.bobbin` files
- `BobbinRuntime`: Main API for games - `from_string()`, `advance()`, `select_choice()`, etc.

## Test Organization

Tests live in `runtime/tests/` with test data in `runtime/tests/cases/`. Test runners use sidecar files:
- `.out` - Expected output lines
- `.trace` - Interactive execution traces (choices, assertions)
- `.err` - Expected error substrings

See `CONTRIBUTING.md` for the trace file format specification.

## Language Syntax

```bobbin
save gold = 100
temp greeted = false
extern player_name

Welcome, {player_name}!
- Buy sword
    set gold = 50
    You bought a sword.
- Leave
    Goodbye!
```

See `docs/language/grammar.md` for the complete EBNF specification.
