#!/bin/bash

# Alea Entropy Client SDK Version Management Script
# This script helps manage semantic versioning for the package

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== Alea Entropy Client SDK Version Management ===${NC}"

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

# Get current version
CURRENT_VERSION=$(node -p "require('./package.json').version")
print_status "Current version: $CURRENT_VERSION"

# Parse command line arguments
BUMP_TYPE=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --major)
            BUMP_TYPE="major"
            shift
            ;;
        --minor)
            BUMP_TYPE="minor"
            shift
            ;;
        --patch)
            BUMP_TYPE="patch"
            shift
            ;;
        --set)
            NEW_VERSION="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--major | --minor | --patch | --set <version>]"
            exit 1
            ;;
    esac
done

if [ -n "$NEW_VERSION" ]; then
    # Set specific version
    print_status "Setting version to: $NEW_VERSION"
    npm version "$NEW_VERSION" --no-git-tag-version
elif [ -n "$BUMP_TYPE" ]; then
    # Bump version according to type
    print_status "Bumping $BUMP_TYPE version..."
    npm version "$BUMP_TYPE" --no-git-tag-version
else
    # Just show current version
    echo "Current version: $CURRENT_VERSION"
    echo "Usage: $0 [--major | --minor | --patch | --set <version>]"
    exit 0
fi

# Get new version
NEW_VERSION=$(node -p "require('./package.json').version")
print_success "Version updated to: $NEW_VERSION"

# Create or update CHANGELOG.md
if [ ! -f "CHANGELOG.md" ]; then
    echo "# Changelog" > CHANGELOG.md
    echo "" >> CHANGELOG.md
fi

# Prepend new version to changelog
TEMP_CHANGELOG=$(mktemp)
echo "# [$NEW_VERSION] - $(date +%Y-%m-%d)" > "$TEMP_CHANGELOG"
echo "" >> "$TEMP_CHANGELOG"
echo "## Added" >> "$TEMP_CHANGELOG"
echo "- " >> "$TEMP_CHANGELOG"
echo "" >> "$TEMP_CHANGELOG"
echo "## Changed" >> "$TEMP_CHANGELOG"
echo "- " >> "$TEMP_CHANGELOG"
echo "" >> "$TEMP_CHANGELOG"
echo "## Fixed" >> "$TEMP_CHANGELOG"
echo "- " >> "$TEMP_CHANGELOG"
echo "" >> "$TEMP_CHANGELOG"
cat CHANGELOG.md >> "$TEMP_CHANGELOG"
mv "$TEMP_CHANGELOG" CHANGELOG.md

print_success "Updated CHANGELOG.md with new version entry"

# If git repository exists, create a tag
if command -v git >/dev/null 2>&1 && git rev-parse --git-dir >/dev/null 2>&1; then
    git add package.json CHANGELOG.md
    git commit -m "chore: bump version to $NEW_VERSION"
    git tag "sdk-v$NEW_VERSION"
    print_success "Created git commit and tag for version $NEW_VERSION"
else
    print_status "Not in a git repository, skipping commit and tag creation"
fi

print_status "Version management completed!"