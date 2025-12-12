---
status: Accepted
date: 2025-01-15
deciders: Phil
---

# Bobbin DSL Compiler Architecture: AST-Based Pipeline with Bytecode VM

## Context and Problem Statement

Bobbin is a domain-specific language for dialogue systems in video games, enabling writers to author branching conversations with jumps between scenes, conditional logic, and variable interpolation. We need to choose a compiler/interpreter architecture that supports:

- Good error reporting with precise source locations
- Editor tooling (LSP, syntax highlighting, linting)
- The control flow patterns inherent to dialogue (jumps, branches, choices)
- Save/load of execution state (for game saves)

How should we structure the compilation and execution pipeline for Bobbin?

## Decision Drivers

- Quality of error messages (mapping runtime errors back to source locations)
- Feasibility of building rich editor extensions (LSP server, linter, formatter, graph visualizer)
- Natural fit for dialogue-specific control flow (jumps between scenes, branching choices)
- Implementation complexity and maintainability
- Ability to pause, serialize, and resume execution (player choices, save games)
- Hot reloading: writers must be able to iterate on dialogue while the game runs

## Considered Options

1. Pure recursive-descent tree-walk interpreter with label registry and continuation loop
2. Two-layer hybrid: flat node registry with state machine executor for scene navigation, tree-walk interpreter for expressions and logic within nodes
3. Direct compilation from tokens to bytecode, executed by a stack-based VM
4. AST-based pipeline: parser produces AST, separate compiler emits bytecode, executed by a stack-based VM

## Decision Outcome

Chosen option: **AST-based pipeline with bytecode VM**, because it provides the best foundation for tooling while cleanly handling dialogue control flow.

The pipeline is:
```
Source → Scanner → Parser → AST → Semantic Analysis → Compiler → Bytecode → VM
```

### Consequences

- Good, because the AST serves as a single source of truth for multiple consumers (compiler, LSP, linter, formatter, graph visualizer)
- Good, because each phase has a single responsibility, making components easier to test and maintain
- Good, because error recovery in the parser can produce partial ASTs, enabling editor tooling to work with broken/incomplete files
- Good, because jumps compile to simple address-setting in the VM, avoiding the complexity of control flow exceptions or continuation management
- Good, because the bytecode VM naturally supports pausing and resuming execution (store program counter + stack + variables)
- Good, because hot reloading is straightforward: recompile source to new bytecode, swap into VM while preserving variable state
- Bad, because it requires more upfront implementation than direct-to-bytecode (estimated ~60% more code for the core compiler)
- Bad, because it uses more memory at compile time (full AST in memory vs streaming compilation)
- Neutral, because the additional compilation passes have negligible performance impact for a DSL of this scale

## Pros and Cons of the Options

### Pure Recursive-Descent Tree-Walk Interpreter

Execute directly from the AST using recursive function calls, with a label registry to resolve jump targets and exception-based or continuation-loop control flow for jumps.

- Good, because it's the simplest architecture conceptually
- Good, because the AST is available for tooling
- Bad, because jumps require awkward control flow mechanisms (exceptions to unwind the call stack, or an outer continuation loop)
- Bad, because pausing execution mid-dialogue is difficult when state is implicit in the call stack
- Bad, because hot reloading requires either restarting execution or complex call stack reconstruction
- Bad, because the execution model fights against the natural structure of dialogue (graph of scenes, not nested tree)

### Two-Layer Hybrid (Flat Node Registry + Tree-Walk)

Parse dialogue into a flat dictionary of scene nodes. An outer state machine loop handles scene-to-scene navigation; an inner tree-walk interpreter evaluates expressions and logic within each scene.

- Good, because scene jumps become trivial (just set the current scene ID)
- Good, because it maps well to how writers think about dialogue (scenes/beats)
- Good, because save/load is straightforward (store current scene ID + variables)
- Good, because hot reloading works at scene granularity: reload scene registry, keep current scene ID + variables
- Bad, because it introduces two distinct execution models that must be understood and maintained
- Bad, because the boundary between "outer" and "inner" interpretation can be awkward for features that span both (e.g., nested conditionals that contain jumps)
- Neutral, because tooling can work with the parsed structure, though it's less uniform than a proper AST

### Direct Token-to-Bytecode Compilation

Single-pass or two-pass compilation directly from token stream to bytecode, skipping the AST. A stack-based VM executes the bytecode.

- Good, because it produces ~20% less code for a minimal working compiler
- Good, because it uses less memory (no AST retained)
- Good, because jumps are trivial in the VM (just set program counter)
- Good, because hot reloading works the same as AST-based: recompile to bytecode, swap into VM
- Bad, because parsing and code generation are interleaved, increasing complexity of each
- Bad, because tooling (LSP, linter) requires a separate parse pass anyway, negating the code savings
- Bad, because error recovery is harder when you can't produce a partial AST

### AST-Based Pipeline with Bytecode VM

Parse to a full AST with source spans on every node. A semantic analysis pass resolves symbols and builds a symbol table. A separate compiler tree-walks the AST to emit bytecode. A stack-based VM executes the bytecode.

- Good, because one parser serves all consumers (compiler, LSP, linter, formatter, visualizer)
- Good, because each phase is simple and independently testable
- Good, because the AST naturally supports error recovery and partial parsing for editor scenarios
- Good, because the symbol table (built during semantic analysis) enables go-to-definition, find-references, and autocomplete
- Good, because jumps are trivial in the VM execution model
- Good, because hot reloading is straightforward: recompile source, swap bytecode into VM while preserving state
- Bad, because it requires more total code than direct-to-bytecode (~60% more for core compiler)
- Bad, because it requires multiple passes over the source

## More Information

The decision was informed by analysis of how dialogue systems differ from general-purpose programming languages. Dialogue is fundamentally a directed graph of scenes with jumps between them, which maps poorly to deeply nested recursive execution but maps naturally to a flat bytecode instruction stream.

For the VM, a minimal instruction set is anticipated: ~15-20 opcodes covering string emission, choices, jumps, conditionals, variable operations, and stack manipulation.

Bytecode will be compiled to memory at load time rather than serialized to disk. Dialogue files are small and compilation is fast, so there's no need for a separate build step or bytecode file format. This keeps source files as the single source of truth and simplifies hot reloading.

Future ADRs may address:

- Localization strategy (string tables, ID generation, runtime locale switching)
- Debug info format for source mapping
- LSP server architecture and incremental parsing strategy
