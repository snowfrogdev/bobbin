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
SAVE    = "save" , " " , identifier , " " , "=" , " " , literal ;
TEMP    = "temp" , " " , identifier , " " , "=" , " " , literal ;
EXTERN  = "extern" , " " , identifier ;
SET     = "set" , " " , identifier , " " , "=" , " " , literal ;
LINE    = text ;                         (* line not starting with "- ", "save ", "temp ", "extern ", or "set " *)
CHOICE  = "-" , " " , text ;             (* line starting with "- " *)
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
interpolation = "{" , identifier , "}" ;
escaped_brace = "{{" | "}}" ;
text_char     = ? any character except "{", "}", and newline ? ;
```

## Notes

### General

- Blank lines are skipped at the lexical level
- Statements execute sequentially; nested statements complete before their parent continues
- Statements are recursive: choices can contain any statements, including other choice sets

### Variable Declarations (`save` and `temp`)

- `save` declares a persistent dialogue global (survives save/load)
- `temp` declares a temporary variable (exists only during execution)
- Both require an initial value
- Both are statically typed: the type is inferred from the initial value
- `temp` types are checked at compile time only
- `save` types are checked at compile time and verified at runtime when reading from storage
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
- Assigning to `temp` or `save` variables is type-checked at compile time
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

- Lines and choice text may contain interpolations: `{variable_name}`
- Use `{{` for a literal `{` character, `}}` for a literal `}`
- Only variable names are currently supported (expressions TBD)
- Example: `Welcome, {player_name}! You have {gold} gold.`

## Future Syntax (TBD)

The following syntax elements are planned but not yet specified:

- **Compound assignment operators**: `+=`, `-=`, `*=`, `/=`
- **Expressions**: Arithmetic, comparison, and logical operators
- **Conditionals**: `if`/`else` structure
- **Tables**: Literal syntax, access syntax, methods
- **Interpolation expressions**: Expressions beyond variable names inside `{...}`
- **Imports**: Module system syntax
- **Commands**: Syntax for triggering game effects (giving items, playing sounds, etc.)
