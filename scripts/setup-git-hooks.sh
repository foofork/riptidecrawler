#!/bin/bash
# Setup git hooks for the project

set -e

HOOKS_DIR=".git/hooks"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "🪝 Setting up git hooks..."
echo ""

# Pre-commit hook (auto-format)
cat > "$HOOKS_DIR/pre-commit" << 'HOOK'
#!/bin/bash
# Pre-commit hook - Auto-format code before commit
# This runs instantly and doesn't block your workflow

set -e

echo "🎨 Auto-formatting staged files..."

# Get list of staged Rust files
STAGED_RS_FILES=$(git diff --cached --name-only --diff-filter=ACM | grep '\.rs$' || true)

if [ -n "$STAGED_RS_FILES" ]; then
    # Format the entire workspace (fast with incremental)
    cargo fmt --all

    # Re-stage the formatted files
    for file in $STAGED_RS_FILES; do
        if [ -f "$file" ]; then
            git add "$file"
        fi
    done

    echo "✅ Code formatted"
else
    echo "ℹ️  No Rust files to format"
fi

echo ""
echo "💡 Tip: Run 'make ci-quick' before pushing for validation"
echo ""

exit 0
HOOK

chmod +x "$HOOKS_DIR/pre-commit"

echo "✅ Pre-commit hook installed (auto-format)"
echo ""
echo "📋 Hook Configuration:"
echo "  ✅ Pre-commit: Auto-format (instant)"
echo "  ❌ Pre-push: Disabled (manual validation)"
echo ""
echo "🚀 Usage:"
echo "  git commit        → Auto-formats code"
echo "  make ci-quick     → Manual quick validation (~30s)"
echo "  make ci           → Manual full validation (~5min)"
echo ""
echo "💡 To disable auto-format temporarily:"
echo "  git commit --no-verify -m 'message'"
echo ""
echo "🎉 Git hooks setup complete!"
