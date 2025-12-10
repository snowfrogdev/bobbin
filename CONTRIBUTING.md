# Contributing to Bobbin

Thanks for wanting to contribute!

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

This creates a junction (Windows) or symlink (Linux/Mac) so that changes to `godot/addons/bobbin/` are immediately reflected in the test project. You only need to run this once per clone.

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
