#!/bin/sh

set -e

HOOKS_DIR=".githooks"

if [ ! -d "$HOOKS_DIR" ]; then
    echo "Error: $HOOKS_DIR not found. Run this script from the project root."
    exit 1
fi

git config core.hooksPath "$HOOKS_DIR"
echo "Git hooks installed: $HOOKS_DIR/pre-commit, $HOOKS_DIR/pre-push"
