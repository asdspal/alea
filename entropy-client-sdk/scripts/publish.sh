#!/bin/bash

# Alea Entropy Client SDK Publishing Script
# This script handles the publishing process for the npm package

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== Alea Entropy Client SDK Publishing Script ===${NC}"

# Function to print status
print_status() {
    echo -e "${YELLOW}[INFO] $1${NC}"
}

# Function to print success
print_success() {
    echo -e "${GREEN}[SUCCESS] $1${NC}"
}

# Function to print error
print_error() {
    echo -e "${RED}[ERROR] $1${NC}"
}

# Check if we're in the right directory
if [ ! -f "package.json" ]; then
    print_error "package.json not found. Please run this script from the client-sdk directory."
    exit 1
fi

# Parse command line arguments
DRY_RUN=false
BUMP_VERSION=false
NEW_VERSION=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --bump-version)
            BUMP_VERSION=true
            shift
            ;;
        --version)
            NEW_VERSION="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--dry-run] [--bump-version] [--version <version>]"
            exit 1
            ;;
    esac
done

# Get current version
CURRENT_VERSION=$(node -p "require('./package.json').version")
print_status "Current version: $CURRENT_VERSION"

# Bump version if requested
if [ "$BUMP_VERSION" = true ]; then
    if [ -n "$NEW_VERSION" ]; then
        print_status "Updating to new version: $NEW_VERSION"
        npm version "$NEW_VERSION" --no-git-tag-version
    else
        print_status "Bumping version (patch)..."
        npm version patch --no-git-tag-version
    fi
    NEW_VERSION=$(node -p "require('./package.json').version")
    print_success "Version updated to: $NEW_VERSION"
fi

# Clean dist directory
print_status "Cleaning dist directory..."
rm -rf dist/

# Run build process
print_status "Building package..."
npm run build

# Check if build was successful
if [ ! -d "dist" ] || [ -z "$(ls -A dist)" ]; then
    print_error "Build failed or dist directory is empty"
    exit 1
fi

print_success "Build completed successfully"

# Run tests before publishing
print_status "Running tests..."
npm test

if [ $? -ne 0 ]; then
    print_error "Tests failed. Cannot proceed with publishing."
    exit 1
fi

print_success "All tests passed"

# Show what will be published (dry run)
print_status "Package contents that will be published:"
if [ "$DRY_RUN" = true ]; then
    npm pack --dry-run
    print_success "Dry run completed. Package would include the above files."
else
    # Check if npm token is available
    if [ -z "$NPM_TOKEN" ]; then
        print_error "NPM_TOKEN environment variable not set. Cannot publish."
        echo "Please set your NPM token: export NPM_TOKEN=<your_token>"
        exit 1
    fi

    # Login to npm (if needed)
    print_status "Checking npm authentication..."
    npm whoami > /dev/null 2>&1 || {
        print_status "Not logged in to npm, attempting to login..."
        echo "//registry.npmjs.org/:_authToken=${NPM_TOKEN}" > .npmrc
    }

    # Publish the package
    print_status "Publishing package to npm..."
    npm publish

    if [ $? -eq 0 ]; then
        print_success "Package published successfully!"
        
        # Create git tag for the new version
        if [ -n "$NEW_VERSION" ]; then
            git tag "sdk-v$NEW_VERSION" 2>/dev/null || echo "Git tag already exists or git not available"
        fi
    else
        print_error "Failed to publish package"
        exit 1
    fi
fi

print_status "Publishing process completed!"