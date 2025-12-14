# Setup script for Windows
# Creates junction to make the Bobbin addon available in the Godot test project

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir

$AddonSrc = Join-Path $ProjectRoot "bindings\godot\addons\bobbin"
$AddonDest = Join-Path $ProjectRoot "test-projects\godot\bobbin-test-project\addons\bobbin"
$AddonsDir = Split-Path -Parent $AddonDest

# Create addons directory if it doesn't exist
if (-not (Test-Path $AddonsDir)) {
    New-Item -ItemType Directory -Path $AddonsDir -Force | Out-Null
}

# Remove existing junction/directory if present
if (Test-Path $AddonDest) {
    # Check if it's a junction/symlink
    $item = Get-Item $AddonDest -Force
    if ($item.Attributes -band [System.IO.FileAttributes]::ReparsePoint) {
        # It's a junction/symlink - remove it
        cmd /c rmdir "$AddonDest"
    } else {
        # It's a regular directory - remove recursively
        Remove-Item -Path $AddonDest -Recurse -Force
    }
}

if ($env:CI -eq "true") {
    # CI: copy (simple, universal)
    Copy-Item -Path $AddonSrc -Destination $AddonDest -Recurse
    Write-Host "Copied addon to test project (CI mode)"
} else {
    # Local dev: junction (no admin required, works like symlink for directories)
    cmd /c mklink /J "$AddonDest" "$AddonSrc"
    Write-Host "Created junction to addon in test project (dev mode)"
}

Write-Host "Done! Addon is now available at: $AddonDest"

# Build Docker image for containerized builds
Write-Host ""
Write-Host "Building Docker image for containerized builds..."
Push-Location $ProjectRoot
docker compose build
Pop-Location
Write-Host "Docker image 'bobbin-build' ready!"