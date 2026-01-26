# Bobbin for VS Code

Language support for [Bobbin](https://github.com/snowfrog/bobbin), a narrative scripting language for branching dialogue and interactive stories in video games.

## Features

### Syntax Highlighting

Full syntax highlighting for `.bobbin` files including:

- Variable declarations (`save`, `temp`, `extern`)
- Assignments (`set`)
- Conditionals (`if`, `elseif`, `else`)
- Logical operators (`and`, `or`, `not`)
- Comparison and arithmetic operators
- Dialogue text and choices
- String interpolations `{variable}` and `{expression}`
- Comments

### Error Diagnostics

Real-time error detection powered by the Bobbin language server:

- Undefined variable references
- Variable shadowing warnings
- Assignment to read-only extern variables
- Parse errors with precise locations

### Autocomplete

Context-aware code completion:

- Variable names (temp, save, extern)
- Keywords (`save`, `temp`, `set`, `extern`, `if`, `elseif`, `else`)
- Boolean literals (`true`, `false`)
- Logical operators (`and`, `or`, `not`)
- Smart filtering inside interpolations `{}`

## Example

```bobbin
save gold = 100
temp greeted = false
extern player_name

Welcome, {player_name}!

- Buy sword (50 gold)
    if gold >= 50
        set gold = gold - 50
        You bought a sword! Gold remaining: {gold}
    else
        You can't afford that.

- Leave
    if greeted
        See you again!
    else
        Goodbye, stranger.
```

## Configuration

| Setting | Description | Default |
|---------|-------------|---------|
| `bobbin.lsp.path` | Custom path to `bobbin-lsp` executable | (bundled) |
| `bobbin.trace.server` | LSP trace level: `off`, `messages`, `verbose` | `off` |

## Commands

| Command | Description |
|---------|-------------|
| `Bobbin: Restart Language Server` | Restart the LSP server |

## Requirements

This extension includes a bundled language server. No additional installation required.

## Links

- [Bobbin Documentation](https://github.com/snowfrog/bobbin)
- [Language Grammar](https://github.com/snowfrog/bobbin/blob/main/docs/language/grammar.md)
- [Report Issues](https://github.com/snowfrog/bobbin/issues)

## License

See [LICENSE.md](LICENSE.md) for details.

---

## Development

For extension development, see [CONTRIBUTING.md](https://github.com/snowfrog/bobbin/blob/main/CONTRIBUTING.md).

### Quick Start

```bash
# Install LSP server
cargo install --path lsp

# Install dependencies and compile
cd editors/vscode
npm install
npm run compile

# Run in VS Code
# Press F5 or select "Run Bobbin Extension" from debug dropdown
```
