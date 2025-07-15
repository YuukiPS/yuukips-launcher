@echo off
REM Verbose build script for YuukiPS Launcher (Batch version)
REM Shows detailed information about what's being compiled

echo === YuukiPS Launcher Verbose Build Script ===
echo Starting build with detailed compilation information...
echo.

REM Change to the Tauri source directory
cd /d "%~dp0src-tauri"

REM Check if we're in the right directory
if not exist "Cargo.toml" (
    echo Error: Cargo.toml not found. Make sure you're in the project root.
    pause
    exit /b 1
)

echo Current directory: %CD%
rustc --version
cargo --version
echo.

REM Set environment variables for verbose output
set CARGO_LOG=cargo::core::compiler::fingerprint=info
set RUST_LOG=info

REM Build with maximum verbosity
echo Starting cargo build with verbose output...
echo This will show detailed information about each crate being compiled.
echo.

REM Use cargo build with verbose flags
cargo build --verbose --message-format=human --timings

REM Check build result
if %ERRORLEVEL% equ 0 (
    echo.
    echo === Build completed successfully! ===
    echo Build artifacts are in: %CD%\target\debug
) else (
    echo.
    echo === Build failed with exit code %ERRORLEVEL% ===
    pause
    exit /b %ERRORLEVEL%
)

REM Show timing information if available
if exist "target\cargo-timings\cargo-timing.html" (
    echo.
    echo Timing report generated: target\cargo-timings\cargo-timing.html
    echo Open this file in a browser to see detailed build timing analysis.
)

echo.
echo === Build script completed ===
pause