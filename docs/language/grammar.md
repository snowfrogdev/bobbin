# Bobbin Grammar

## Syntax Grammar

```ebnf
script     = { block } ;
block      = line | choice_set ;
line       = LINE , NEWLINE ;
choice_set = choice , { choice } ;
choice     = CHOICE , NEWLINE , [ INDENT , { block } , DEDENT ] ;
```

## Lexical Grammar

```ebnf
LINE    = text ;                         (* line not starting with "- " *)
CHOICE  = "-" , " " , text ;             (* line starting with "- " *)
NEWLINE = "\n" | "\r\n" | "\r" ;
INDENT  = ? increase in indentation level ? ;
DEDENT  = ? decrease in indentation level ? ;
text    = { char }+ ;
char    = ? any character except newline ? ;
```

## Notes

### General

- Blank lines are skipped at the lexical level
- Blocks execute sequentially; nested blocks complete before their parent continues
- Blocks are recursive: choices can contain any blocks, including other choice sets

### Choices

- Space required after `-` for choices (i.e., the `"-␣"` prefix)
- A LINE is any line not starting with `"-␣"`
- A CHOICE is any line starting with `"-␣"`, with the text after the prefix as its content

### Indentation

- Only spaces are allowed for indentation (tabs are forbidden)
- Indent level is determined by the number of leading spaces
- Sibling blocks must use the same indentation level
- No fixed number of spaces per level is required, but consistency is enforced
