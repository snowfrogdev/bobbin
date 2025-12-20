# Contributing to Bobbin

Thanks for wanting to contribute!

## Prerequisites

- [Docker Desktop](https://www.docker.com/products/docker-desktop/)

## Development Setup

After cloning the repo, run the setup script to link the Bobbin addon into the Godot test project:

**Windows (PowerShell):**

```powershell
.\scripts\setup.ps1
```

**Linux/Mac:**

```bash
chmod +x scripts/setup.sh
./scripts/setup.sh
```

This creates a junction (Windows) or symlink (Linux/Mac) so that changes to `godot/addons/bobbin/` are immediately reflected in the test project, and builds the Docker image for containerized builds. You only need to run this once per clone.

## Building with Docker

From the repo root:

```bash
# Local development (fast builds)
docker compose run --rm --build godot windows
docker compose run --rm --build godot windows debug
docker compose run --rm --build godot all              # all platforms × all types

# All platforms, single build type
docker compose run --rm --build godot all release
docker compose run --rm --build godot all debug

# CI builds (optimized, smaller binaries)
docker compose run --rm --build godot all --ci
```

Artifacts are automatically copied to `bindings/godot/addons/bobbin/bin/`.

### Build Options

| Target | Debug | Release | Notes |
|--------|-------|---------|-------|
| windows | `.debug.dll` | `.dll` | Cross-compiled via mingw-w64 |
| linux | `.debug.so` | `.so` | Native Linux build |
| wasm | `.wasm` | `.wasm` | Debug only; fixed name required by gdext |
| all | All targets | All targets | Builds all platforms × all types by default |

| Flag | Effect |
|------|--------|
| `--ci` | Use optimized profiles (slower build, smaller binaries) |

**Notes:**

- By default, builds use fast profiles for quick local iteration. Use `--ci` for optimized release builds.
- macOS builds are not supported by the Docker container (cross-compiling for macOS requires Apple SDK). macOS builds will be handled by CI/CD on native macOS runners.
- WASM release builds are not yet supported due to nightly toolchain requirements.

## Testing

### Godot Integration Tests

Open `test-projects/godot/bobbin-test-project/` in Godot Editor.

### Runtime Unit Tests

```bash
cargo test -p bobbin-runtime
```

## Runtime Development

### Test Organization

Runtime tests live in `runtime/tests/` with this structure:

- `basic.rs` — Simple dialogue tests
- `choices.rs` — Choice/branching tests
- `variables.rs` — Variable and interpolation tests
- `syntax.rs` — Syntax error tests
- `support/` — Test utilities and runners
- `cases/` — Test case data files organized by feature

Run a specific test category:

```bash
cargo test -p bobbin-runtime --test choices
```

### Adding a New Test

1. Create a `.bobbin` file in the appropriate `cases/` subdirectory
2. Create a sidecar file with expected output:
   - `.out` for linear output tests (one expected line per line)
   - `.trace` for interactive tests with choices (see format below)
   - `.err` for error tests (substrings that must appear in error message)
3. Add a `#[test]` function in the corresponding test file (e.g., `choices.rs`)

### Sidecar File Formats

**`.out`** — Expected output lines (one per line):

```text
Hello, World!
How are you?
```

**`.trace`** — Execution trace with named paths for interactive tests:

```trace
--- path: select_first
> How are you?
[advance]
? Good | Bad
[choice 0]
! done

--- path: select_second
> How are you?
[advance]
[choice 1]
! done
```

Format elements:

- `--- path: <name>` — Start named execution path
- `> <text>` — Assert `current_line()` equals text
- `? A | B` — Assert `current_choices()` equals `["A", "B"]`
- `! done` — Assert `has_more()` is false
- `! has_more` — Assert `has_more()` is true
- `! waiting_for_choice` — Assert `is_waiting_for_choice()` is true
- `[advance]` — Call `advance()`
- `[choice <n>]` — Call `select_choice(n)`
- `# comment` — Comment (ignored)

**`.err`** — Substrings that must appear in error message (one per line, case-insensitive):

```text
undefined
name
```

## Releasing

### Godot Addon

Releases are automated via GitHub Actions. There are two ways to trigger a release:

**Option 1: Tag push (recommended)**

```bash
# Update version in bindings/godot/Cargo.toml, then:
git tag godot-addon-v1.0.0
git push origin godot-addon-v1.0.0
```

**Option 2: Manual dispatch**

1. Go to Actions → "Release Godot Addon"
2. Click "Run workflow"
3. Enter the version (e.g., `1.0.0`)
4. Click "Run workflow"

The workflow builds for Windows, Linux, macOS, and WASM, then creates a GitHub Release with the addon zip attached.

### Version Conventions

- Use semantic versioning: `MAJOR.MINOR.PATCH`
- Tag format: `{component}-v{version}` (e.g., `godot-addon-v1.0.0`)
- Each distributable is versioned independently

## Before you submit

By submitting a pull request, you agree that:

- You have the right to submit the code (it's yours, or you have permission)
- Your contribution will be incorporated into the project and distributed under the Bobbin License
- You retain copyright to your contribution
- You grant Snowfrog Studio a non-exclusive, perpetual, worldwide license to use,
  modify, relicense, or sublicense your contribution as part of the Bobbin project —
  including for commercial purposes

This keeps the project consistently licensed and allows for future possibilities such as
commercial arrangements with studios.

See `LICENSE.md` for the full project license terms.
