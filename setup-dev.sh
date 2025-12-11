#!/bin/bash
# Quick setup script for Tabrela development environment

set -e

echo "🚀 Tabrela Development Setup"
echo "=============================="
echo ""

# Check for pre-commit
if ! command -v pre-commit &> /dev/null; then
    echo "⚠️  pre-commit not found. Install it with:"
    echo "   pip install pre-commit"
    echo ""
    read -p "Install pre-commit now? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        pip install pre-commit
    fi
fi

# Install pre-commit hooks
if command -v pre-commit &> /dev/null; then
    echo "📦 Installing pre-commit hooks..."
    pre-commit install
    echo "✓ Pre-commit hooks installed"
else
    echo "⊘ Skipping pre-commit setup"
fi

# Check for cargo-audit
if ! command -v cargo-audit &> /dev/null; then
    echo ""
    echo "⚠️  cargo-audit not found (optional but recommended)"
    echo "   Install with: cargo install cargo-audit"
fi

echo ""
echo "✓ Setup complete!"
echo ""
echo "Next steps:"
echo "  1. Configure your .env files"
echo "  2. Run: ./run_dev.sh"
echo ""
echo "For more info, see README_DEVELOPMENT.md"
