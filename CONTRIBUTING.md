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
# Release builds (default)
docker compose run --rm --build godot windows
docker compose run --rm --build godot linux
docker compose run --rm --build godot wasm
docker compose run --rm --build godot all

# Debug builds
docker compose run --rm --build godot windows debug
docker compose run --rm --build godot linux debug
docker compose run --rm --build godot all debug
```

Artifacts are automatically copied to `bindings/godot/addons/bobbin/bin/`.

### Build Options

| Target | Debug | Release | Notes |
|--------|-------|---------|-------|
| windows | `.debug.dll` | `.dll` | Cross-compiled via mingw-w64 |
| linux | `.debug.so` | `.so` | Native Linux build |
| wasm | `.wasm` | `.wasm` | Debug only; fixed name required by gdext |
| all | All targets | All targets | Builds everything |

**Notes:**

- macOS builds are not supported by the Docker container (cross-compiling for macOS requires Apple SDK). macOS builds will be handled by CI/CD on native macOS runners.
- WASM release builds are not yet supported due to nightly toolchain requirements.

## Testing

Open `test-projects/godot/bobbin-test-project/` in Godot Editor.

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
