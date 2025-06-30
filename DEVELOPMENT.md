# Development Guide & Best Practices

## 🔧 Platform-Specific Dependencies

### Fixed Issues

✅ **Cross-Platform Build Compatibility**
- Moved `winreg` and `registry` crates to Windows-specific dependencies
- Fixed Ubuntu 20.04 deprecation by upgrading to Ubuntu 22.04
- Properly configured conditional compilation for platform-specific code

### Dependency Management Best Practices

```toml
# ✅ Correct: Platform-specific dependencies
[target.'cfg(windows)'.dependencies]
winapi = { version = "0.3", features = ["processthreadsapi", "securitybaseapi", "winnt", "handleapi"] }
winreg = "0.52"
registry = "1.2"

[target.'cfg(unix)'.dependencies]
# Add Unix-specific dependencies here if needed

# ❌ Incorrect: Platform-specific crates in general dependencies
[dependencies]
winreg = "0.52"  # This will fail on Linux!
```

## 🚀 Enhanced Build System

### 1. **Automated Quality Checks**

Add to your GitHub Actions workflow:

```yaml
# Add before the build step
- name: Security audit
  run: cargo audit
  continue-on-error: true

- name: Lint Rust code
  run: cargo clippy -- -D warnings

- name: Format check
  run: cargo fmt --check

- name: Frontend linting
  run: npm run lint

- name: Type checking
  run: npx tsc --noEmit
```

### 2. **Build Optimization**

```yaml
# Enhanced caching strategy
- name: Cache dependencies
  uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/registry
      ~/.cargo/git
      src-tauri/target
      node_modules
    key: ${{ runner.os }}-deps-${{ hashFiles('**/Cargo.lock', '**/package-lock.json') }}
    restore-keys: |
      ${{ runner.os }}-deps-

# Parallel builds
- name: Build with parallel jobs
  run: cargo build --release --jobs $(nproc)
  env:
    CARGO_BUILD_JOBS: 4
```

### 3. **Multi-Architecture Support**

```yaml
# Future enhancement: ARM64 support
matrix:
  include:
    - platform: 'ubuntu-22.04'
      target: 'x86_64-unknown-linux-gnu'
      name: 'Linux-x64'
    - platform: 'ubuntu-22.04'
      target: 'aarch64-unknown-linux-gnu'
      name: 'Linux-ARM64'
    - platform: 'windows-latest'
      target: 'x86_64-pc-windows-msvc'
      name: 'Windows-x64'
    - platform: 'windows-latest'
      target: 'aarch64-pc-windows-msvc'
      name: 'Windows-ARM64'
```

## 🔒 Security Enhancements

### 1. **Dependency Scanning**

```yaml
# Add to workflow
- name: Run Trivy vulnerability scanner
  uses: aquasecurity/trivy-action@master
  with:
    scan-type: 'fs'
    scan-ref: '.'
    format: 'sarif'
    output: 'trivy-results.sarif'

- name: Upload Trivy scan results
  uses: github/codeql-action/upload-sarif@v2
  with:
    sarif_file: 'trivy-results.sarif'
```

### 2. **Code Signing Setup**

```yaml
# For production releases
- name: Import Code Signing Certificate
  if: matrix.platform == 'windows-latest' && github.event_name == 'release'
  run: |
    echo "${{ secrets.WINDOWS_CERTIFICATE }}" | base64 --decode > certificate.p12
    
- name: Sign Windows executable
  if: matrix.platform == 'windows-latest' && github.event_name == 'release'
  run: |
    signtool sign /f certificate.p12 /p "${{ secrets.CERTIFICATE_PASSWORD }}" /tr http://timestamp.digicert.com /td sha256 /fd sha256 "src-tauri/target/release/bundle/msi/*.msi"
```

## 📊 Performance Monitoring

### 1. **Build Time Tracking**

```yaml
- name: Track build performance
  run: |
    echo "Build started at: $(date)"
    time cargo build --release
    echo "Build completed at: $(date)"
    
- name: Analyze binary size
  run: |
    ls -lah src-tauri/target/release/
    du -sh src-tauri/target/release/bundle/*
```

### 2. **Bundle Size Analysis**

```yaml
- name: Bundle size report
  run: |
    echo "## Bundle Sizes" >> $GITHUB_STEP_SUMMARY
    echo "| Platform | Format | Size |" >> $GITHUB_STEP_SUMMARY
    echo "|----------|--------|------|" >> $GITHUB_STEP_SUMMARY
    
    if [ -d "src-tauri/target/release/bundle/msi" ]; then
      SIZE=$(du -sh src-tauri/target/release/bundle/msi/*.msi | cut -f1)
      echo "| Windows | MSI | $SIZE |" >> $GITHUB_STEP_SUMMARY
    fi
    
    if [ -d "src-tauri/target/release/bundle/deb" ]; then
      SIZE=$(du -sh src-tauri/target/release/bundle/deb/*.deb | cut -f1)
      echo "| Linux | DEB | $SIZE |" >> $GITHUB_STEP_SUMMARY
    fi
```

## 🧪 Testing Strategy

### 1. **Automated Testing**

```yaml
# Add comprehensive testing
- name: Run Rust tests
  run: cargo test --all-features
  
- name: Run frontend tests
  run: npm test
  
- name: Integration tests
  run: |
    # Start the app in test mode
    npm run tauri:dev &
    sleep 10
    # Run your integration tests here
    npm run test:integration
```

### 2. **Cross-Platform Testing**

```yaml
# Test on multiple Node.js versions
strategy:
  matrix:
    node-version: [18, 20, 22]
    platform: [ubuntu-22.04, windows-latest, macos-latest]
```

## 📝 Code Quality Standards

### 1. **Rust Code Standards**

```toml
# Add to Cargo.toml
[lints.rust]
unused_imports = "warn"
unused_variables = "warn"
dead_code = "warn"

[lints.clippy]
all = "warn"
pedantic = "warn"
nursery = "warn"
```

### 2. **TypeScript Standards**

```json
// Add to package.json scripts
{
  "scripts": {
    "lint:fix": "eslint . --fix",
    "type-check": "tsc --noEmit",
    "format": "prettier --write .",
    "pre-commit": "npm run lint && npm run type-check && npm run format"
  }
}
```

## 🔄 Continuous Integration Improvements

### 1. **Matrix Strategy Enhancement**

```yaml
strategy:
  fail-fast: false
  matrix:
    include:
      - platform: 'ubuntu-22.04'
        target: 'x86_64-unknown-linux-gnu'
        name: 'Linux'
        test: true
      - platform: 'windows-latest'
        target: 'x86_64-pc-windows-msvc'
        name: 'Windows'
        test: true
      - platform: 'macos-latest'
        target: 'universal-apple-darwin'
        name: 'macOS'
        test: false  # Skip tests on macOS for now
```

### 2. **Environment-Specific Builds**

```yaml
# Different configurations for different environments
- name: Build for development
  if: github.ref != 'refs/heads/main'
  run: npm run tauri:build -- --debug
  
- name: Build for production
  if: github.ref == 'refs/heads/main'
  run: npm run tauri:build
  env:
    TAURI_PRIVATE_KEY: ${{ secrets.TAURI_PRIVATE_KEY }}
    TAURI_KEY_PASSWORD: ${{ secrets.TAURI_KEY_PASSWORD }}
```

## 📋 Maintenance Checklist

### Weekly
- [ ] Check for dependency updates with `npm audit` and `cargo audit`
- [ ] Review build performance metrics
- [ ] Monitor bundle sizes for regressions

### Monthly
- [ ] Update GitHub Actions versions
- [ ] Review and update Rust toolchain
- [ ] Check for new Tauri releases
- [ ] Update Node.js version if needed

### Quarterly
- [ ] Review platform support (Ubuntu LTS versions)
- [ ] Evaluate new build optimizations
- [ ] Update security scanning tools
- [ ] Review code signing certificates

## 🚨 Troubleshooting Common Issues

### Platform-Specific Build Failures

```bash
# Check platform-specific dependencies
cargo tree --target x86_64-unknown-linux-gnu
cargo tree --target x86_64-pc-windows-msvc

# Verify conditional compilation
cargo check --target x86_64-unknown-linux-gnu
cargo check --target x86_64-pc-windows-msvc
```

### Dependency Conflicts

```bash
# Clean and rebuild
cargo clean
npm ci
cargo build

# Check for duplicate dependencies
cargo tree --duplicates
```

### Build Cache Issues

```bash
# Clear all caches
cargo clean
rm -rf node_modules
rm package-lock.json
npm install
```

This development guide ensures your build system is robust, secure, and maintainable across all supported platforms.