# Bobbin VS Code Extension

Language support for Bobbin narrative scripts (`.bobbin` files).

## Features

- Syntax highlighting
- Error diagnostics (undefined variables, parse errors, etc.)

## Development Setup

### Prerequisites

1. **Install the LSP server:**
   ```bash
   cargo install --path lsp
   ```

2. **Install npm dependencies:**
   ```bash
   cd editors/vscode
   npm install
   ```

3. **Compile TypeScript:**
   ```bash
   npm run compile
   ```

### Running the Extension

#### Option A: Extension Development Host (temporary)

Press `F5` from VS Code with the `editors/vscode` folder open, or select "Run Bobbin Extension" from the debug dropdown if you have the root `bobbin` folder open.

#### Option B: Install in your VS Code (persistent)

Link the extension into your VS Code extensions folder:

**Windows (run in CMD as regular user):**
```cmd
mklink /J "%USERPROFILE%\.vscode\extensions\bobbin-vscode" "d:\path\to\bobbin\editors\vscode"
```

**macOS/Linux:**
```bash
ln -s /path/to/bobbin/editors/vscode ~/.vscode/extensions/bobbin-vscode
```

Then reload VS Code (`Ctrl+Shift+P` → "Reload Window").

### Rebuilding After Changes

If you modify the extension TypeScript code:
```bash
cd editors/vscode
npm run compile
```

Then reload VS Code to pick up changes.

If you modify the LSP server Rust code:
```bash
cargo install --path lsp
```

Then reload VS Code (the extension will restart the LSP server).
