#!/bin/bash
# Build script for YuukiPS Launcher (Linux/macOS)

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
GRAY='\033[0;37m'
NC='\033[0m' # No Color

# Default values
TARGET="local"
CLEAN=false
DEV=false
HELP=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -t|--target)
            TARGET="$2"
            shift 2
            ;;
        -c|--clean)
            CLEAN=true
            shift
            ;;
        -d|--dev)
            DEV=true
            shift
            ;;
        -h|--help)
            HELP=true
            shift
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

show_help() {
    echo -e "${GREEN}YuukiPS Launcher Build Script${NC}"
    echo ""
    echo -e "${YELLOW}Usage: ./build.sh [options]${NC}"
    echo ""
    echo -e "${YELLOW}Options:${NC}"
    echo -e "  ${NC}-t, --target <target>    Build target (local, windows, linux, all)${NC}"
    echo -e "  ${NC}-c, --clean             Clean build artifacts before building${NC}"
    echo -e "  ${NC}-d, --dev               Run in development mode${NC}"
    echo -e "  ${NC}-h, --help              Show this help message${NC}"
    echo ""
    echo -e "${YELLOW}Examples:${NC}"
    echo -e "  ${GRAY}./build.sh                    # Build for current platform${NC}"
    echo -e "  ${GRAY}./build.sh -t windows         # Build for Windows${NC}"
    echo -e "  ${GRAY}./build.sh -t linux           # Build for Linux${NC}"
    echo -e "  ${GRAY}./build.sh -t all             # Build for all platforms${NC}"
    echo -e "  ${GRAY}./build.sh -d                 # Run development server${NC}"
    echo -e "  ${GRAY}./build.sh -c                 # Clean and build${NC}"
}

check_prerequisites() {
    echo -e "${BLUE}Checking prerequisites...${NC}"
    
    # Check Node.js
    if command -v node &> /dev/null; then
        NODE_VERSION=$(node --version)
        echo -e "${GREEN}✓ Node.js: $NODE_VERSION${NC}"
    else
        echo -e "${RED}✗ Node.js not found. Please install Node.js 18+${NC}"
        exit 1
    fi
    
    # Check Rust
    if command -v rustc &> /dev/null; then
        RUST_VERSION=$(rustc --version)
        echo -e "${GREEN}✓ Rust: $RUST_VERSION${NC}"
    else
        echo -e "${RED}✗ Rust not found. Please install Rust${NC}"
        exit 1
    fi
    
    # Check system dependencies for Linux
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        echo -e "${BLUE}Checking Linux dependencies...${NC}"
        
        MISSING_DEPS=()
        
        if ! dpkg -l | grep -q libwebkit2gtk-4.0-dev; then
            MISSING_DEPS+=("libwebkit2gtk-4.0-dev")
        fi
        
        if ! dpkg -l | grep -q libappindicator3-dev; then
            MISSING_DEPS+=("libappindicator3-dev")
        fi
        
        if ! dpkg -l | grep -q librsvg2-dev; then
            MISSING_DEPS+=("librsvg2-dev")
        fi
        
        if ! dpkg -l | grep -q patchelf; then
            MISSING_DEPS+=("patchelf")
        fi
        
        if [ ${#MISSING_DEPS[@]} -gt 0 ]; then
            echo -e "${YELLOW}Missing dependencies: ${MISSING_DEPS[*]}${NC}"
            echo -e "${YELLOW}Installing missing dependencies...${NC}"
            sudo apt-get update
            sudo apt-get install -y "${MISSING_DEPS[@]}"
        fi
        
        echo -e "${GREEN}✓ Linux dependencies satisfied${NC}"
    fi
}

install_dependencies() {
    echo -e "${BLUE}Installing dependencies...${NC}"
    npm ci
    echo -e "${GREEN}✓ Dependencies installed${NC}"
}

clean_build() {
    echo -e "${BLUE}Cleaning build artifacts...${NC}"
    
    if [ -d "dist" ]; then
        rm -rf "dist"
    fi
    
    if [ -d "src-tauri/target" ]; then
        rm -rf "src-tauri/target"
    fi
    
    echo -e "${GREEN}✓ Build artifacts cleaned${NC}"
}

build_app() {
    local build_target=$1
    
    echo -e "${BLUE}Building for target: $build_target${NC}"
    
    case $build_target in
        "local")
            npm run tauri:build
            ;;
        "windows")
            npm run tauri:build -- --target x86_64-pc-windows-msvc
            ;;
        "linux")
            npm run tauri:build -- --target x86_64-unknown-linux-gnu
            ;;
        "all")
            echo -e "${YELLOW}Building for all platforms...${NC}"
            npm run tauri:build -- --target x86_64-pc-windows-msvc
            npm run tauri:build -- --target x86_64-unknown-linux-gnu
            ;;
        *)
            echo -e "${RED}Unknown target: $build_target${NC}"
            exit 1
            ;;
    esac
    
    echo -e "${GREEN}✓ Build completed successfully${NC}"
}

start_dev() {
    echo -e "${BLUE}Starting development server...${NC}"
    npm run tauri:dev
}

show_build_artifacts() {
    echo -e "${BLUE}Build artifacts:${NC}"
    
    local bundle_path="src-tauri/target/release/bundle"
    if [ -d "$bundle_path" ]; then
        find "$bundle_path" -name "*.msi" -o -name "*.exe" -o -name "*.deb" -o -name "*.AppImage" | while read -r file; do
            local size=$(du -h "$file" | cut -f1)
            echo -e "  ${GRAY}$file ($size)${NC}"
        done
    fi
}

# Main execution
if [ "$HELP" = true ]; then
    show_help
    exit 0
fi

echo -e "${GREEN}YuukiPS Launcher Build Script${NC}"
echo -e "${GREEN}==============================${NC}"
echo ""

check_prerequisites

if [ "$DEV" = true ]; then
    install_dependencies
    start_dev
    exit 0
fi

install_dependencies

if [ "$CLEAN" = true ]; then
    clean_build
fi

build_app "$TARGET"
show_build_artifacts

echo ""
echo -e "${GREEN}Build completed! Check the artifacts above.${NC}"