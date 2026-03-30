#!/usr/bin/env bash
set -e

# Read version from Cargo.toml
VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
TAG="v$VERSION"

echo "Version: $TAG"

# Check if tag already exists
if git rev-parse "$TAG" >/dev/null 2>&1; then
    echo "Error: tag $TAG already exists. Please bump the version in Cargo.toml first."
    exit 1
fi

# Check for uncommitted changes
if ! git diff --quiet || ! git diff --cached --quiet; then
    echo "Error: you have uncommitted changes. Please commit first."
    exit 1
fi

git tag "$TAG"
git push origin "$TAG"

echo "Tag $TAG pushed. GitHub Actions will now build and publish the release."
