#!/usr/bin/env pwsh
# Build script for YuukiPS Launcher
# Supports Windows and Linux (via PowerShell Core)

param(
    [string]$Target = "local",
    [switch]$Clean,
    [switch]$Dev,
    [switch]$Help
)

function Show-Help {
    Write-Host "YuukiPS Launcher Build Script" -ForegroundColor Green
    Write-Host ""
    Write-Host "Usage: .\build.ps1 [options]" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Options:" -ForegroundColor Yellow
    Write-Host "  -Target <target>    Build target (local, windows, linux, all)" -ForegroundColor White
    Write-Host "  -Clean             Clean build artifacts before building" -ForegroundColor White
    Write-Host "  -Dev               Run in development mode" -ForegroundColor White
    Write-Host "  -Help              Show this help message" -ForegroundColor White
    Write-Host ""
    Write-Host "Examples:" -ForegroundColor Yellow
    Write-Host "  .\build.ps1                    # Build for current platform" -ForegroundColor Gray
    Write-Host "  .\build.ps1 -Target windows    # Build for Windows" -ForegroundColor Gray
    Write-Host "  .\build.ps1 -Target linux      # Build for Linux" -ForegroundColor Gray
    Write-Host "  .\build.ps1 -Target all        # Build for all platforms" -ForegroundColor Gray
    Write-Host "  .\build.ps1 -Dev               # Run development server" -ForegroundColor Gray
    Write-Host "  .\build.ps1 -Clean             # Clean and build" -ForegroundColor Gray
}

function Test-Prerequisites {
    Write-Host "Checking prerequisites..." -ForegroundColor Blue
    
    # Check Node.js
    try {
        $nodeVersion = node --version
        Write-Host "✓ Node.js: $nodeVersion" -ForegroundColor Green
    } catch {
        Write-Host "✗ Node.js not found. Please install Node.js 18+" -ForegroundColor Red
        exit 1
    }
    
    # Check Rust
    try {
        $rustVersion = rustc --version
        Write-Host "✓ Rust: $rustVersion" -ForegroundColor Green
    } catch {
        Write-Host "✗ Rust not found. Please install Rust" -ForegroundColor Red
        exit 1
    }
    
    # Check Tauri CLI
    try {
        $tauriVersion = npx tauri --version
        Write-Host "✓ Tauri CLI: $tauriVersion" -ForegroundColor Green
    } catch {
        Write-Host "✗ Tauri CLI not found. Installing..." -ForegroundColor Yellow
        npm install -g @tauri-apps/cli
    }
}

function Install-Dependencies {
    Write-Host "Installing dependencies..." -ForegroundColor Blue
    npm ci
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Failed to install dependencies" -ForegroundColor Red
        exit 1
    }
    Write-Host "✓ Dependencies installed" -ForegroundColor Green
}

function Clean-Build {
    Write-Host "Cleaning build artifacts..." -ForegroundColor Blue
    
    if (Test-Path "dist") {
        Remove-Item -Recurse -Force "dist"
    }
    
    if (Test-Path "src-tauri\target") {
        Remove-Item -Recurse -Force "src-tauri\target"
    }
    
    Write-Host "✓ Build artifacts cleaned" -ForegroundColor Green
}

function Build-App {
    param([string]$BuildTarget)
    
    Write-Host "Building for target: $BuildTarget" -ForegroundColor Blue
    
    switch ($BuildTarget) {
        "local" {
            npm run tauri:build
        }
        "windows" {
            npm run tauri:build -- --target x86_64-pc-windows-msvc
        }
        "linux" {
            npm run tauri:build -- --target x86_64-unknown-linux-gnu
        }
        "all" {
            Write-Host "Building for all platforms..." -ForegroundColor Yellow
            npm run tauri:build -- --target x86_64-pc-windows-msvc
            npm run tauri:build -- --target x86_64-unknown-linux-gnu
        }
        default {
            Write-Host "Unknown target: $BuildTarget" -ForegroundColor Red
            exit 1
        }
    }
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Build failed" -ForegroundColor Red
        exit 1
    }
    
    Write-Host "✓ Build completed successfully" -ForegroundColor Green
}

function Start-Dev {
    Write-Host "Starting development server..." -ForegroundColor Blue
    npm run tauri:dev
}

function Show-BuildArtifacts {
    Write-Host "Build artifacts:" -ForegroundColor Blue
    
    $bundlePath = "src-tauri\target\release\bundle"
    if (Test-Path $bundlePath) {
        Get-ChildItem -Recurse $bundlePath -Include "*.msi", "*.exe", "*.deb", "*.AppImage" | ForEach-Object {
            $size = [math]::Round($_.Length / 1MB, 2)
            Write-Host "  $($_.FullName) ($size MB)" -ForegroundColor Gray
        }
    }
}

# Main execution
if ($Help) {
    Show-Help
    exit 0
}

Write-Host "YuukiPS Launcher Build Script" -ForegroundColor Green
Write-Host "==============================" -ForegroundColor Green
Write-Host ""

Test-Prerequisites

if ($Dev) {
    Install-Dependencies
    Start-Dev
    exit 0
}

Install-Dependencies

if ($Clean) {
    Clean-Build
}

Build-App -BuildTarget $Target
Show-BuildArtifacts

Write-Host ""
Write-Host "Build completed! Check the artifacts above." -ForegroundColor Green