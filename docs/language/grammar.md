# Bobbin Grammar

## Syntax Grammar

```ebnf
script      = { statement } ;
statement   = save_decl | temp_decl | extern_decl | assignment | line | choice_set ;
save_decl   = SAVE , NEWLINE ;
temp_decl   = TEMP , NEWLINE ;
extern_decl = EXTERN , NEWLINE ;
assignment  = SET , NEWLINE ;
line        = LINE , NEWLINE ;
choice_set  = choice , { choice } ;
choice      = CHOICE , NEWLINE , [ INDENT , { statement } , DEDENT ] ;
```

## Lexical Grammar

```ebnf
SAVE    = "save" , identifier , "=" , literal ;
TEMP    = "temp" , identifier , "=" , literal ;
EXTERN  = "extern" , identifier ;
SET     = "set" , identifier , "=" , literal ;
LINE    = text ;                         (* line not starting with "- ", "save ", "temp ", "extern ", or "set " *)
CHOICE  = "-" , text ;                   (* line starting with "- " *)
NEWLINE = "\n" | "\r\n" | "\r" ;
INDENT  = ? increase in indentation level ? ;
DEDENT  = ? decrease in indentation level ? ;

identifier = letter , { letter | digit | "_" } ;
literal    = number | string | boolean ;
number     = [ "-" ] , digit , { digit } , [ "." , digit , { digit } ] ;
string     = '"' , { string_char } , '"' ;
string_char = ? any character except '"' and newline, or escaped character ? ;
boolean    = "true" | "false" ;

letter = "a" | ... | "z" | "A" | ... | "Z" ;
digit  = "0" | ... | "9" ;

text          = { text_segment }+ ;
text_segment  = text_char | interpolation | escaped_brace ;
interpolation = "{" , expression , "}" ;
expression    = logical_or ;
logical_or    = logical_and , { "or" , logical_and } ;
logical_and   = equality , { "and" , equality } ;
equality      = comparison , { ( "==" | "!=" ) , comparison } ;
comparison    = term , { ( "<" | "<=" | ">" | ">=" ) , term } ;
term          = factor , { ( "+" | "-" ) , factor } ;
factor        = unary , { ( "*" | "/" | "%" ) , unary } ;
unary         = ( "-" | "not" ) , unary | primary ;
primary       = identifier | literal | "(" , expression , ")" ;
escaped_brace = "{{" | "}}" ;
text_char     = ? any character except "{", "}", and newline ? ;
```

## Notes

### General

- Blank lines are skipped at the lexical level
- Statements execute sequentially; nested statements complete before their parent continues
- Statements are recursive: choices can contain any statements, including other choice sets
- Whitespace between tokens is handled by the scanner; the lexical grammar shows logical structure only

### String Literals

String literals support these escape sequences:

- `\n` - newline
- `\t` - tab
- `\r` - carriage return
- `\"` - double quote
- `\\` - backslash

### Variable Declarations (`save` and `temp`)

- `save` declares a persistent dialogue global (survives save/load)
- `temp` declares a temporary variable (exists only during execution)
- Both require an initial value
- Type is inferred from the initial value
- See ADR-0002 for the state management architecture
- See ADR-0004 for the type system and storage architecture

### Host Variable Declarations (`extern`)

- `extern` declares that a variable is provided by the host application
- No initial value: the host owns and provides the value at runtime
- Read-only from Bobbin's perspective; `set` on extern variables is a semantic error
- Must be declared at top level, before first use
- Dynamically typed: the type is discovered at runtime when the host provides the value
- Duplicate declarations in the same file are errors; across files they are allowed (idempotent)
- If the host doesn't provide a declared extern variable at runtime, a runtime error occurs
- See ADR-0004 for the two-interface architecture

### Assignments

- `set` modifies an existing variable
- The variable must be declared with `save` or `temp`
- Assigning to `extern` variables is a semantic error (they are read-only)
- See ADR-0003 for the syntax decision rationale

### Choices

- Space required after `-` for choices (i.e., the `"-␣"` prefix)
- A LINE is any line not starting with `"-␣"`, `"save "`, `"temp "`, `"extern "`, or `"set "`
- A CHOICE is any line starting with `"-␣"`, with the text after the prefix as its content

### Indentation

- Only spaces are allowed for indentation (tabs are forbidden)
- Indent level is determined by the number of leading spaces
- Sibling statements must use the same indentation level
- No fixed number of spaces per level is required, but consistency is enforced

### Interpolation

- Lines and choice text may contain interpolations: `{expression}`
- Use `{{` for a literal `{` character, `}}` for a literal `}`
- **Simple interpolation**: `{variable_name}` displays the variable's value
- **Arithmetic expressions**: `{x + y}`, `{x - y}`, `{x * y}`, `{x / y}`, `{x % y}`
  - Both operands must be **numbers** (strings and booleans are not allowed)
  - Division and modulo by zero produce a runtime error
  - Unary negation: `{-x}` negates a numeric value
  - Parentheses for grouping: `{(a + b) * c}`
- **Operator precedence** (lowest to highest):
  1. `or` (logical OR)
  2. `and` (logical AND)
  3. `==`, `!=` (equality)
  4. `<`, `<=`, `>`, `>=` (comparison)
  5. `+`, `-` (addition, subtraction)
  6. `*`, `/`, `%` (multiplication, division, modulo)
  7. `not`, unary `-` (logical NOT, negation)
  8. `()` (parentheses)
- **Equality expressions**: `{x == y}` and `{x != y}`
  - Both operands can be variables or literals (symmetric expressions)
  - Supported forms: `{var == var}`, `{var == literal}`, `{literal == var}`, `{literal == literal}`
  - Result is `true` or `false` (displayed as text)
  - Both operands must have the same type (type mismatch is a semantic error)
- **Ordering expressions**: `{x < y}`, `{x <= y}`, `{x > y}`, `{x >= y}`
  - Both operands must be **numbers** (strings and booleans are not allowed)
  - Result is `true` or `false` (displayed as text)
  - Comparing non-numeric types is a semantic error
- **Logical expressions**: `{a and b}`, `{a or b}`, `{not a}`
  - All operands must be **booleans** (numbers and strings are not allowed)
  - `and` returns `true` if both operands are `true`
  - `or` returns `true` if at least one operand is `true`
  - `not` inverts the boolean value
  - Both operands are always evaluated (no short-circuit evaluation)
  - Note: `and`, `or`, `not` are reserved keywords in interpolation contexts
- Examples:
  - `Welcome, {player_name}! You have {gold} gold.`
  - `Damage dealt: {base_damage * multiplier}`
  - `Gold after purchase: {gold - 100}` or `Double damage: {damage * 2}`
  - `Remainder: {count % 3}` or `Negated: {-score}`
  - `Is ready: {count == 10}` or `Different names: {name != "Bob"}`
  - `Is ten: {10 == count}` (literal on left side)
  - `Always true: {true == true}` (literal-to-literal comparison)
  - `Low health: {health < 10}` or `Enough gold: {gold >= 100}`
  - `Combined: {(2 + 3) * 4}` → `20`
  - `Ready for battle: {is_brave and has_sword}`
  - `Can proceed: {is_healthy or has_potion}`
  - `Not tired: {not is_tired}`
  - `In range: {x > 5 and x < 10}`
  - `Override precedence: {(false or true) and true}` → `true`

## Future Syntax (TBD)

The following syntax elements are planned but not yet specified:

- **Compound assignment operators**: `+=`, `-=`, `*=`, `/=`
- **Conditionals**: `if`/`else` structure
- **Tables**: Literal syntax, access syntax, methods
- **Imports**: Module system syntax
- **Commands**: Syntax for triggering game effects (giving items, playing sounds, etc.)
