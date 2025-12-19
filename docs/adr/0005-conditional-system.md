---
status: Accepted
date: 2025-12-18
deciders: Phil
---

# Conditional System Design

## Context and Problem Statement

Bobbin needs conditional content display to allow dialogue to branch based on game state. This ADR captures the design decisions for:

1. **Block conditional syntax**: How `if/elif/else` statements are structured
2. **Expression syntax**: Operators for comparisons and boolean logic
3. **Type behavior**: Truthiness rules and cross-type comparisons
4. **Conditional choices**: How conditions interact with the choice system
5. **Error handling**: How invalid conditions are reported

## Decision Drivers

- **Writer-friendly**: Non-programmers should find the syntax intuitive
- **Consistency**: New syntax should align with existing Bobbin patterns (indentation, choices)
- **Simplicity**: Avoid features that add complexity without clear benefit
- **Predictability**: Behavior should be unsurprising and easy to reason about

## Decision Outcome

### Block Conditional Structure

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Block termination | Indentation-based | Consistent with existing choice syntax |
| Else-if keyword | `elif` | Concise, single token, widely known from Python |
| Negative conditionals | No `unless` | `if not` is sufficient and unambiguous |

**Example syntax:**

```bobbin
if gold >= 100:
    You're wealthy!
elif gold >= 10:
    You have some money.
else:
    You're broke.
```

### Expression Syntax

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Condition delimiter | Colon after condition (`if x:`) | Pairs naturally with indentation blocks |
| Comparison operators | C-style (`==`, `!=`, `<`, `>`, `<=`, `>=`) | Unambiguous; `=` already used for assignment |
| Logical operators | Word-based (`and`, `or`, `not`) | Writer-friendly, reads like English |
| Operator precedence | Standard (not > arithmetic > comparison > and > or) | Matches Python/C/JavaScript expectations |
| Parentheses | Allowed for grouping | Essential for complex conditions |

**Precedence table (highest to lowest):**

| Level | Operators |
|-------|-----------|
| 1 (highest) | `not` |
| 2 | `*`, `/`, `%` |
| 3 | `+`, `-` |
| 4 | `<`, `>`, `<=`, `>=` |
| 5 | `==`, `!=` |
| 6 | `and` |
| 7 (lowest) | `or` |

### Truthiness and Type Coercion

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Truthiness | Only `false` is falsy | Simple, explicit; use `== 0` or `== ""` for explicit checks |
| Type comparison | Strict, no coercion | Different types return `false` for equality; ordering between incompatible types is a runtime error |

This means:
- `0` is truthy (use `if count == 0:` to check for zero)
- `""` is truthy (use `if name == "":` to check for empty string)
- `1 == "1"` returns `false` (no type coercion)
- `"hello" > 5` is a runtime error (incompatible types for ordering)

### Conditional Choices

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Condition placement | After marker, before text (`- {cond} Text`) | Concise, prominent, follows Ink's pattern |
| Condition syntax | Abbreviated (no `if` keyword) | Context distinguishes from interpolation |
| Unavailable choice behavior | Host decides | Runtime provides availability state; host chooses presentation |
| Conditionals in choice bodies | Yes | Natural composition of statements |
| Choices in conditionals | Yes | Follows from recursive grammar |

**Example syntax:**

```bobbin
- {gold >= 10} Buy potion
- {reputation > 50} Ask for a discount
- Leave empty-handed
```

The runtime provides each choice with an `available` flag. The host application decides whether to hide unavailable choices or show them greyed out.

Block syntax is also supported for grouping multiple conditional choices:

```bobbin
if is_member:
    - Access VIP area
    - Get member discount
- Regular option
```

### Error Handling

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Undefined variable | Compile-time error | Consistent with interpolation; resolver catches it |
| Type mismatch (ordering) | Runtime error | `"hello" > 5` fails at runtime |
| Type mismatch (equality) | Returns `false` | `1 == "1"` is `false`, not an error |
| Missing extern | Runtime error | Consistent with existing `MissingExternVariable` |

### Features Not Included (Deferred)

The following were considered but deferred:

- **Inline conditional text**: `{gold > 0: "some" | "no"}` - use separate lines instead
- **Interpolation expressions**: `{gold + 10}` - only variables allowed in `{...}` for now
- **Switch/case statements**: `if/elif/else` is sufficient
- **Pattern matching**: Range checks use `if x >= 1 and x <= 10:`
- **Guard clauses on choices**: Host decides behavior based on availability state

## Consequences

- Good, because syntax is consistent with existing Bobbin patterns
- Good, because word-based operators are accessible to non-programmers
- Good, because strict truthiness prevents subtle bugs (0 and "" being truthy)
- Good, because host control over unavailable choices enables diverse UX patterns
- Good, because compile-time errors catch typos early
- Neutral, because runtime errors for type mismatches require testing to catch
- Bad, because no inline conditional text means more verbose dialogue for small variations

## Related Decisions

- ADR-0001: Compiler Architecture (establishes compilation pipeline)
- ADR-0002: Variable and State Management (defines variable categories)
- ADR-0004: Variable Storage Architecture (defines HostState interface used in conditions)
