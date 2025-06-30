# Build Instructions

## Automated Builds with GitHub Actions

This project includes an automated build system using GitHub Actions that creates setup files for both Windows and Linux platforms.

### How it works

The GitHub Actions workflow (`.github/workflows/build.yml`) automatically:

1. **Triggers on**:
   - Push to `main` or `master` branch
   - New tags (for releases)
   - Pull requests
   - Manual workflow dispatch

2. **Builds for**:
   - **Windows**: Creates `.msi` and `.exe` installers
   - **Linux**: Creates `.deb` packages and `.AppImage` files

3. **Artifacts**: All build artifacts are uploaded and available for download

4. **Releases**: When you push a tag (e.g., `v1.0.0`), it automatically creates a GitHub release with all installers

### Creating a Release

To create a new release:

```bash
# Tag your commit
git tag v1.0.0
git push origin v1.0.0
```

The workflow will automatically:
- Build for all platforms
- Create installers/packages
- Create a GitHub release
- Upload all files to the release

### Manual Build Trigger

You can also manually trigger builds:
1. Go to the "Actions" tab in your GitHub repository
2. Select "Build and Release" workflow
3. Click "Run workflow"

## Local Development Builds

### Prerequisites

**Windows:**
- Node.js 18+
- Rust (latest stable)
- Visual Studio Build Tools

**Linux (Ubuntu 22.04+ recommended):**
```bash
sudo apt-get update
sudo apt-get install -y libwebkit2gtk-4.0-dev libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
```

**Note:** The build system uses Ubuntu 22.04 in CI/CD as Ubuntu 20.04 will be deprecated on April 15, 2025.

### Build Commands

```bash
# Install dependencies
npm install

# Development build
npm run tauri:dev

# Production build
npm run tauri:build
```

### Build Outputs

After running `npm run tauri:build`, you'll find:

**Windows:**
- `src-tauri/target/release/bundle/msi/` - MSI installer
- `src-tauri/target/release/bundle/nsis/` - NSIS installer

**Linux:**
- `src-tauri/target/release/bundle/deb/` - Debian package
- `src-tauri/target/release/bundle/appimage/` - AppImage

## Troubleshooting

### Common Issues

1. **Build fails on Linux**: Make sure all system dependencies are installed
2. **Windows build fails**: Ensure Visual Studio Build Tools are properly installed
3. **Node.js issues**: Clear cache with `npm ci` instead of `npm install`
4. **Rust compilation errors**: Update Rust with `rustup update`

### Getting Help

If you encounter issues:
1. Check the GitHub Actions logs for detailed error messages
2. Ensure all dependencies are correctly installed
3. Try building locally first to isolate the issue

## Customization

### Modifying Build Targets

To add more platforms or modify build settings, edit `.github/workflows/build.yml`:

```yaml
matrix:
  include:
    - platform: 'macos-latest'  # Add macOS
      args: '--target universal-apple-darwin'
      name: 'macOS'
      extension: ''
```

### Release Configuration

Modify the release settings in `src-tauri/tauri.conf.json` under the `bundle` section to customize:
- App icons
- Installer languages
- Bundle identifiers
- Installation options