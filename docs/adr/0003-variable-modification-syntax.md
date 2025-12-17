---
status: Accepted
date: 2025-01-16
deciders: Phil
---

# Variable Modification Syntax

## Context and Problem Statement

Bobbin dialogue files contain both narrative text and code that modifies state. The parser must distinguish between:

```
The merchant smiles.          # Dialogue text
gold = 50                     # Is this dialogue or code?
You bought the sword.         # Dialogue text
```

How should writers indicate that a line is code (an assignment) rather than dialogue?

## Decision Drivers

- **Parser simplicity**: Prefer a context-free grammar where the parser doesn't need external information (like a symbol table) to parse correctly
- **Clear visual distinction**: Readers should easily identify code vs. dialogue when scanning a file
- **Familiar syntax**: Minimize learning curve for writers
- **Minimal parser changes**: Build on existing scanner/parser architecture
- **Error quality**: Typos and mistakes should produce clear error messages

## Considered Options

1. **Bare assignment**: No prefix, parser infers from context
   ```
   gold = 50
   has_sword = true
   ```

2. **`set` keyword prefix**: Explicit keyword marks assignments
   ```
   set gold = 50
   set has_sword = true
   ```

3. **Tilde prefix** (Ink-style): Single character marks assignments
   ```
   ~ gold = 50
   ~ has_sword = true
   ```

4. **Curly brace blocks**: Code wrapped in braces
   ```
   { gold = 50 }
   {
       gold = 50
       has_sword = true
   }
   ```

## Decision Outcome

Chosen option: **`set` keyword prefix**

```
set gold = 50
set has_sword = true
```

### Implementation

The scanner recognizes lines starting with `set ` and emits a `Set` token. The parser then handles assignments without needing to consult a symbol table.

```rust
// Scanner (simplified)
if line.starts_with("set ") {
    Token::Set(line[4..].to_string())
} else if line.starts_with("- ") {
    Token::Choice(line[2..].to_string())
} else {
    Token::Line(line)
}
```

This follows the existing pattern where `- ` marks choices and plain lines are dialogue.

### Consequences

- Good, because the grammar remains context-free (parser doesn't need symbol table)
- Good, because assignments are visually distinct when scanning a file
- Good, because the pattern matches existing syntax (`- ` for choices, `set ` for assignments)
- Good, because implementation requires minimal changes (~40 lines vs ~100+ for alternatives)
- Good, because typos in variable names produce clear errors ("unknown variable 'gld'")
- Bad, because it's slightly more verbose than bare assignment (4 extra characters per line)
- Neutral, because the syntax is similar to Yarn Spinner's `<<set>>` but without angle brackets

## Pros and Cons of the Options

### Bare Assignment

```
gold = 50
```

- Good, because it's the most minimal syntax
- Good, because it's familiar from Python and other languages
- Bad, because it requires context-sensitive parsing (need symbol table at parse time)
- Bad, because it couples the parser to the resolver (architectural concern)
- Bad, because ambiguous cases exist (dialogue that looks like `x = 5`)
- Bad, because implementation requires ~100+ lines and two-pass parsing or declaration-before-use

**Parser complexity analysis**: To distinguish `gold = 50` (assignment) from `Hello there` (dialogue), the parser must know that `gold` is a declared variable. This requires either:
1. Two-pass parsing (collect declarations first, then parse)
2. Strict declaration-before-use ordering
3. Symbol table accessible during parsing

All of these add significant complexity and couple the parser to the resolver.

### Tilde Prefix (Ink-style)

```
~ gold = 50
~ has_sword = true
```

- Good, because it's concise (1 character)
- Good, because Ink users will recognize it
- Bad, because `~` is an unfamiliar symbol for non-Ink users
- Bad, because it's not obviously "code" to narrative designers
- Neutral, because implementation complexity is similar to `set`

### Curly Brace Blocks

```
{ gold = 50 }
{
    gold = 50
    has_sword = true
}
```

- Good, because braces clearly indicate "this is code"
- Good, because multiple statements can be grouped
- Bad, because it potentially conflicts with `{expression}` interpolation syntax
- Bad, because it's more verbose for single statements
- Bad, because mixing brace-based code with indentation-based structure is inconsistent

**Conflict analysis**: If `{expression}` is used for interpolation in dialogue:
```
You have {gold} gold.  # Interpolation
{ gold = 50 }          # Code block
```

The parser must distinguish based on context (inline vs. standalone), adding complexity.

## More Information

### Declaration Syntax

Variable declarations use separate keywords and don't need the `set` prefix:

```
save merchant_relationship = 0   # Declaration (persistent)
temp counter = 0                  # Declaration (temporary)
set merchant_relationship = 10    # Assignment (modification)
```

This distinction is intentional:
- `save`/`temp` create variables (declarations)
- `set` modifies existing variables (assignments)

### Future Considerations

- **Compound assignment**: Operators like `+=`, `-=` are TBD and may be added later
- **Expression syntax**: The right-hand side of `set` currently only supports literals; full expressions are TBD

### Related Decisions

- ADR-0001: Compiler Architecture (establishes the scanner/parser structure)
- ADR-0002: Variable and State Management (covers the three-tier variable model)
