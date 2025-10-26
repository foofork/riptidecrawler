#!/bin/bash
# Code Examples Validator
# Extracts and validates bash/curl code blocks from markdown files

set -eo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Counters
total_blocks=0
valid_blocks=0
invalid_blocks=0
warnings=0

# Error collection
declare -a errors=()
declare -a warning_msgs=()

# Find project root (assuming script is in docs/tests/)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "Code Examples Validator"
echo "======================="
echo "Project Root: $PROJECT_ROOT"
echo ""

# Function to validate bash syntax
validate_bash() {
    local code="$1"
    local file="$2"
    local line_num="$3"

    # Skip comments and empty lines
    if [[ "$code" =~ ^[[:space:]]*# ]] || [[ -z "$code" ]]; then
        return 0
    fi

    # Check for common errors

    # 1. Missing quotes in curl commands with JSON
    if [[ "$code" =~ curl ]] && [[ "$code" =~ -d[[:space:]]*\{ ]] && [[ ! "$code" =~ -d[[:space:]]*[\'\"] ]]; then
        errors+=("$file:$line_num: Error: curl -d flag with unquoted JSON")
        return 1
    fi

    # 2. Invalid JSON structure (basic check for unquoted keys)
    if [[ "$code" =~ \{[[:space:]]*[a-zA-Z_][a-zA-Z0-9_]*[[:space:]]*: ]]; then
        errors+=("$file:$line_num: Error: JSON with unquoted keys detected")
        return 1
    fi

    # 3. Dangerous commands without safeguards
    if [[ "$code" =~ rm[[:space:]]+-rf[[:space:]]+/ ]] || [[ "$code" =~ rm[[:space:]]+-rf[[:space:]]+\* ]]; then
        errors+=("$file:$line_num: Error: Dangerous rm -rf command detected")
        return 1
    fi

    # 4. Missing semicolon or && after complex commands
    if [[ "$code" =~ \&\&[[:space:]]*$ ]] || [[ "$code" =~ \|[[:space:]]*$ ]]; then
        warning_msgs+=("$file:$line_num: Warning: Command ends with && or | (incomplete)")
        ((warnings++))
    fi

    return 0
}

# Function to extract and validate code blocks from a markdown file
validate_markdown_file() {
    local file="$1"
    local in_code_block=0
    local code_type=""
    local code_content=""
    local code_start_line=0
    local current_line=0

    while IFS= read -r line || [ -n "$line" ]; do
        ((current_line++))

        # Check for code block start
        if [[ "$line" =~ ^\`\`\`(bash|sh|shell|curl|json) ]]; then
            in_code_block=1
            code_type="${BASH_REMATCH[1]}"
            code_content=""
            code_start_line=$current_line
            ((total_blocks++))
        # Check for code block end
        elif [[ "$line" =~ ^\`\`\`[[:space:]]*$ ]] && [[ $in_code_block -eq 1 ]]; then
            in_code_block=0

            # Validate the code block
            if [[ "$code_type" =~ ^(bash|sh|shell|curl)$ ]]; then
                # Process each line of the code block
                local code_line_valid=1
                while IFS= read -r code_line || [ -n "$code_line" ]; do
                    if ! validate_bash "$code_line" "$file" "$code_start_line"; then
                        code_line_valid=0
                    fi
                done <<< "$code_content"

                if [[ $code_line_valid -eq 1 ]]; then
                    ((valid_blocks++))
                else
                    ((invalid_blocks++))
                fi
            elif [[ "$code_type" == "json" ]]; then
                # Validate JSON - simple check for balanced braces
                local open_count=$(echo "$code_content" | tr -cd '{' | wc -c)
                local close_count=$(echo "$code_content" | tr -cd '}' | wc -c)

                if [[ $open_count -eq $close_count ]] && [[ $open_count -gt 0 ]]; then
                    ((valid_blocks++))
                else
                    errors+=("$file:$code_start_line: Error: Invalid JSON syntax (unbalanced braces)")
                    ((invalid_blocks++))
                fi
            else
                ((valid_blocks++))  # Unknown type, assume valid
            fi

            code_type=""
            code_content=""
        # Accumulate code content
        elif [[ $in_code_block -eq 1 ]]; then
            code_content+="$line"$'\n'
        fi
    done < "$file"
}

# Find all markdown files in docs directory and key README files
echo "Scanning for markdown files..."
file_count=0

# Process docs directory
if [[ -d "$PROJECT_ROOT/docs" ]]; then
    while IFS= read -r file; do
        ((file_count++))
        echo "  [$file_count] Processing: ${file#$PROJECT_ROOT/}"
        validate_markdown_file "$file"
    done < <(find "$PROJECT_ROOT/docs" -type f -name "*.md" 2>/dev/null || true)
fi

# Process main README
if [[ -f "$PROJECT_ROOT/README.md" ]]; then
    ((file_count++))
    echo "  [$file_count] Processing: README.md"
    validate_markdown_file "$PROJECT_ROOT/README.md"
fi

# Process SDK directory
if [[ -d "$PROJECT_ROOT/sdk" ]]; then
    while IFS= read -r file; do
        ((file_count++))
        echo "  [$file_count] Processing: ${file#$PROJECT_ROOT/}"
        validate_markdown_file "$file"
    done < <(find "$PROJECT_ROOT/sdk" -type f -name "*.md" 2>/dev/null | grep -v node_modules || true)
fi

echo ""
echo "Validation Results"
echo "=================="
echo "Files processed: $file_count"
echo "Total code blocks found: $total_blocks"
echo -e "${GREEN}Valid blocks: $valid_blocks${NC}"

if [[ $invalid_blocks -gt 0 ]]; then
    echo -e "${RED}Invalid blocks: $invalid_blocks${NC}"
fi

if [[ $warnings -gt 0 ]]; then
    echo -e "${YELLOW}Warnings: $warnings${NC}"
fi

# Print errors
if [[ ${#errors[@]} -gt 0 ]]; then
    echo ""
    echo -e "${RED}Errors:${NC}"
    for error in "${errors[@]}"; do
        echo -e "${RED}  ✗ $error${NC}"
    done
fi

# Print warnings
if [[ ${#warning_msgs[@]} -gt 0 ]]; then
    echo ""
    echo -e "${YELLOW}Warnings:${NC}"
    for warning in "${warning_msgs[@]}"; do
        echo -e "${YELLOW}  ⚠ $warning${NC}"
    done
fi

echo ""

# Exit with error if there are invalid blocks
if [[ $invalid_blocks -gt 0 ]]; then
    echo -e "${RED}Validation FAILED${NC}"
    exit 1
else
    echo -e "${GREEN}Validation PASSED${NC}"
    exit 0
fi
