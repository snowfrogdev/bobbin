# Bobbin Grammar

## Syntax Grammar

```ebnf
script     = { block } ;
block      = line | choice_set ;
line       = STRING , NEWLINE ;
choice_set = choice , { choice } ;
choice     = CHOICE , NEWLINE , [ INDENT , { block } , DEDENT ] ;
```

## Lexical Grammar

```ebnf
STRING  = { char }+ ;
CHOICE  = "-" , " " , { char } ;
NEWLINE = "\n" | "\r\n" | "\r" ;
INDENT  = ? increase in indentation level ? ;
DEDENT  = ? decrease in indentation level ? ;
char    = ? any character except newline ? ;
```

## Notes

- Blank lines are skipped at the lexical level
- Space required after `-` for choices
- STRING is any line not starting with `- `
