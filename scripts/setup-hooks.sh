#!/bin/bash
# Setup git hooks for RipTide development

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
HOOKS_DIR="$REPO_ROOT/.git/hooks"

echo "========================================="
echo "Setting up RipTide Git Hooks"
echo "========================================="
echo ""

# Create pre-push hook
echo "Installing pre-push hook..."
cat > "$HOOKS_DIR/pre-push" << 'EOF'
#!/bin/bash

# Git LFS check
command -v git-lfs >/dev/null 2>&1 || {
    printf >&2 "\n%s\n\n" "This repository is configured for Git LFS but 'git-lfs' was not found on your path. If you no longer wish to use Git LFS, remove this hook by deleting the 'pre-push' file in the hooks directory (set by 'core.hookspath'; usually '.git/hooks')."
    exit 2
}
git lfs pre-push "$@"

# Quality gate enforcement for main branch
remote="$1"
url="$2"

# Read from stdin: <local ref> <local sha1> <remote ref> <remote sha1>
while read local_ref local_sha remote_ref remote_sha
do
    # Check if pushing to main branch
    if [[ "$remote_ref" == "refs/heads/main" ]]; then
        # Check what files are being pushed
        changed_files=$(git diff --name-only "$remote_sha" "$local_sha" 2>/dev/null || git diff --name-only HEAD~1 HEAD)

        # Check if any Rust code files in crates/ changed
        code_changed=$(echo "$changed_files" | grep -E '^crates/.*\.(rs|toml)$' || true)

        if [[ -n "$code_changed" ]]; then
            echo ""
            echo "========================================="
            echo "Pre-push quality gate for main branch"
            echo "Code changes detected - running quality gate"
            echo "========================================="
            echo ""

            # Run quality gate script
            if ! ./scripts/quality_gate.sh; then
                echo ""
                echo "✗ Quality gate FAILED - push to main blocked"
                echo "  Fix the issues above and try again."
                echo ""
                exit 1
            fi

            echo ""
            echo "✓ Quality gate passed - proceeding with push to main"
            echo ""
        else
            echo ""
            echo "========================================="
            echo "Pre-push check for main branch"
            echo "No code changes detected (docs/scripts only)"
            echo "Skipping quality gate - proceeding with push"
            echo "========================================="
            echo ""
        fi
    fi
done

exit 0
EOF

chmod +x "$HOOKS_DIR/pre-push"
echo "✓ Pre-push hook installed"

echo ""
echo "========================================="
echo "Git hooks setup complete!"
echo ""
echo "The pre-push hook will:"
echo "  - Run quality gate when pushing code to main"
echo "  - Skip quality gate for docs/scripts-only changes"
echo "  - Enforce: format, clippy, build, tests"
echo "========================================="
