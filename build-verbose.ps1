#!/usr/bin/env pwsh
# Verbose build script for YuukiPS Launcher
# Shows detailed information about what's being compiled

Write-Host "=== YuukiPS Launcher Verbose Build Script ===" -ForegroundColor Green
Write-Host "Starting build with detailed compilation information..." -ForegroundColor Yellow
Write-Host ""

# Change to the Tauri source directory
Set-Location "src-tauri"

# Check if we're in the right directory
if (-not (Test-Path "Cargo.toml")) {
    Write-Host "Error: Cargo.toml not found. Make sure you're in the project root." -ForegroundColor Red
    exit 1
}

Write-Host "Current directory: $(Get-Location)" -ForegroundColor Cyan
Write-Host "Rust version: $(rustc --version)" -ForegroundColor Cyan
Write-Host "Cargo version: $(cargo --version)" -ForegroundColor Cyan
Write-Host ""

# Set environment variables for verbose output
$env:CARGO_LOG = "cargo::core::compiler::fingerprint=info"
$env:RUST_LOG = "info"

# Build with maximum verbosity
Write-Host "Starting cargo build with verbose output..." -ForegroundColor Green
Write-Host "This will show detailed information about each crate being compiled." -ForegroundColor Yellow
Write-Host ""

# Use cargo build with verbose flags
cargo build --verbose --message-format=human --timings

# Check build result
if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "=== Build completed successfully! ===" -ForegroundColor Green
    Write-Host "Build artifacts are in: $(Get-Location)\target\debug" -ForegroundColor Cyan
} else {
    Write-Host ""
    Write-Host "=== Build failed with exit code $LASTEXITCODE ===" -ForegroundColor Red
    exit $LASTEXITCODE
}

# Show timing information if available
if (Test-Path "target\cargo-timings\cargo-timing.html") {
    Write-Host ""
    Write-Host "Timing report generated: target\cargo-timings\cargo-timing.html" -ForegroundColor Cyan
    Write-Host "Open this file in a browser to see detailed build timing analysis." -ForegroundColor Yellow
}

Write-Host ""
Write-Host "=== Build script completed ===" -ForegroundColor Green