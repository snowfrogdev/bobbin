# Bobbin Grammar

## Syntax Grammar

```ebnf
script        = { statement } ;
statement     = save_decl | temp_decl | extern_decl | extern_command_decl
              | assignment | command_call | line | choice_set | if_stmt ;
save_decl     = SAVE , NEWLINE ;
temp_decl     = TEMP , NEWLINE ;
extern_decl   = EXTERN , NEWLINE ;
extern_command_decl = EXTERN_COMMAND , NEWLINE ;
assignment    = SET , NEWLINE ;
command_call  = COMMAND_CALL , NEWLINE ;
line          = LINE , NEWLINE ;
choice_set    = choice , { choice } ;
choice        = CHOICE , NEWLINE , [ INDENT , { statement } , DEDENT ] ;

if_stmt       = IF , NEWLINE , INDENT , statement , { statement } , DEDENT ,
                { elseif_clause } ,
                [ else_clause ] ;
elseif_clause = ELSEIF , NEWLINE , INDENT , statement , { statement } , DEDENT ;
else_clause   = ELSE , NEWLINE , INDENT , statement , { statement } , DEDENT ;
```

## Lexical Grammar

```ebnf
SAVE           = "save" , identifier , "=" , expression ;
TEMP           = "temp" , identifier , "=" , expression ;
EXTERN         = "extern" , identifier ;
EXTERN_COMMAND = "extern" , identifier , "(" , [ param_list ] , ")" ;
SET            = "set" , identifier , "=" , expression ;
COMMAND_CALL   = identifier , "(" , [ arg_list ] , ")" ;
IF             = "if" , expression ;            (* condition must evaluate to boolean *)
ELSEIF         = "elseif" , expression ;        (* condition must evaluate to boolean *)
ELSE           = "else" ;
LINE           = text ;                         (* line not starting with "- ", "save ", "temp ", "extern ", "set ", "if ", "elseif ", "else", or command call *)
CHOICE         = "-" , text ;                   (* line starting with "- " *)

param_list     = identifier , { "," , identifier } ;
arg_list       = expression , { "," , expression } ;
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
- Both require an initial expression (can be a literal or computed value)
- Type is inferred from the initial expression's result type
- Expressions can reference previously declared variables: `temp y = x + 5`
- Forward references are not allowed: `temp x = y` fails if `y` isn't declared yet
- See ADR-0002 for the state management architecture
- See ADR-0004 for the type system and storage architecture

#### Save Variable Expression Semantics

Save variables with expressions are initialized **once** at first run:

```bobbin
save health = base_health + bonus  // Evaluated on first run only
```

On subsequent loads, the variable value is restored from save storage,
and the initializer expression is NOT re-evaluated.

Extern variables in initializers are evaluated at initialization time.
If extern state changes between save and load, the saved value takes precedence.

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

- `set` modifies an existing variable with an expression
- The variable must be declared with `save` or `temp`
- Expressions can reference other variables: `set health = health - damage`
- Self-referential assignments are allowed: `set counter = counter + 1`
- Assigning to `extern` variables is a semantic error (they are read-only)
- See ADR-0003 for the syntax decision rationale

### Commands

Commands allow Bobbin scripts to trigger game effects (giving items, playing sounds, etc.). Commands are fire-and-forget: they do not return values.

#### Command Declaration

Commands must be declared before use with `extern` followed by a parameter list:

```bobbin
extern give_gold(amount)       // Single parameter
extern give_item(name, count)  // Multiple parameters
extern save_game()             // No parameters
```

- The parameter names are for documentation only; they are not enforced at runtime
- The number of parameters determines the expected arity (argument count)
- Command names must not conflict with variable names

#### Command Invocation

Commands are invoked with function-call syntax:

```bobbin
give_gold(100)
give_item("sword", 1)
save_game()
```

- Arguments are expressions (variables, literals, arithmetic, etc.)
- Argument count must match the declaration's parameter count
- Arguments are evaluated left-to-right before the command executes

#### Complete Example

```bobbin
extern player_name
extern player_gold
extern give_gold(amount)
extern play_sound(name)

Welcome, {player_name}! You have {player_gold} gold.

- Buy potion (50 gold)
    give_gold(-50)
    play_sound("purchase")
    Here's your potion!
- Leave
    Goodbye!
```

#### Error Handling

- Undeclared command invocation: compile-time error
- Wrong number of arguments: compile-time error
- Runtime command failures are reported to the game engine, which decides how to handle them

See ADR-0005 for the design rationale.

### Choices

- Space required after `-` for choices (i.e., the `"-␣"` prefix)
- A LINE is any line not starting with `"-␣"`, `"save "`, `"temp "`, `"extern "`, `"set "`, `"if "`, `"elseif "`, or `"else"`
- A CHOICE is any line starting with `"-␣"`, with the text after the prefix as its content

### Conditionals

Bobbin supports conditional execution with `if`, `elseif`, and `else`:

- `if <expression>` - executes block if expression evaluates to `true`
- `elseif <expression>` - checked if all previous conditions were `false`
- `else` - executes if all previous conditions were `false`
- Blocks are indentation-delimited (same as choices)
- Empty blocks are not allowed (must contain at least one statement)
- Condition expressions must evaluate to boolean (type mismatch is a semantic error)
- Conditionals can nest inside choices and vice versa

Example:

```bobbin
temp health = 25

if health < 10
    You're dying!
elseif health < 50
    You're wounded.
else
    You're healthy!
```

Conditionals with choices:

```bobbin
save gold = 100

if gold >= 50
    You can afford items.
    - Buy sword (50 gold)
        set gold = 50
        You bought a sword!
    - Leave
        Goodbye.
else
    You can't afford anything.
```

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
- **Tables**: Literal syntax, access syntax, methods
- **Imports**: Module system syntax
