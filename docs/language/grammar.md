# Bobbin Grammar

## Syntax Grammar

```ebnf
script     = { statement } ;
statement  = save_decl | temp_decl | assignment | line | choice_set ;
save_decl  = SAVE , NEWLINE ;
temp_decl  = TEMP , NEWLINE ;
assignment = SET , NEWLINE ;
line       = LINE , NEWLINE ;
choice_set = choice , { choice } ;
choice     = CHOICE , NEWLINE , [ INDENT , { statement } , DEDENT ] ;
```

## Lexical Grammar

```ebnf
SAVE    = "save" , " " , identifier , " " , "=" , " " , literal ;
TEMP    = "temp" , " " , identifier , " " , "=" , " " , literal ;
SET     = "set" , " " , identifier , " " , "=" , " " , literal ;
LINE    = text ;                         (* line not starting with "- ", "save ", "temp ", or "set " *)
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

### Variable Declarations

- `save` declares a persistent dialogue global (survives save/load)
- `temp` declares a temporary variable (exists only during execution)
- Both require an initial value
- See ADR-0002 for the state management architecture

### Assignments

- `set` modifies an existing variable
- The variable must be declared (`save` or `temp`) or provided by the game
- See ADR-0003 for the syntax decision rationale

### Choices

- Space required after `-` for choices (i.e., the `"-␣"` prefix)
- A LINE is any line not starting with `"-␣"`, `"save "`, `"temp "`, or `"set "`
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
