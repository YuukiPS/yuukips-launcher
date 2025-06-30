const fs = require('fs');
const path = require('path');

// Read package.json to get the current version
const packagePath = path.join(__dirname, 'package.json');
const pkg = JSON.parse(fs.readFileSync(packagePath, 'utf8'));
const newVersion = pkg.version;

console.log(`Syncing version to: ${newVersion}`);

// Update Cargo.toml
const cargoPath = path.join(__dirname, 'src-tauri', 'Cargo.toml');
let cargoContent = fs.readFileSync(cargoPath, 'utf8');

// Replace the version line in Cargo.toml
cargoContent = cargoContent.replace(/^version = "[^"]*"/m, `version = "${newVersion}"`);

// Write back to Cargo.toml
fs.writeFileSync(cargoPath, cargoContent);

console.log(`Updated Cargo.toml version to: ${newVersion}`);