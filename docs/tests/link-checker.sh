#!/bin/bash
# Link Checker
# Validates internal links and anchors in markdown files

set -eo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Counters
total_links=0
valid_links=0
broken_links=0
total_anchors=0
valid_anchors=0
broken_anchors=0

# Error collection
declare -a errors=()
declare -A file_anchors  # Maps file paths to their available anchors

# Find project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "Link Checker"
echo "============"
echo "Project Root: $PROJECT_ROOT"
echo ""

# Function to extract anchors from a markdown file
extract_anchors() {
    local file="$1"
    local anchors=""

    # Extract headers and convert to anchor format
    while IFS= read -r line; do
        if [[ "$line" =~ ^#{1,6}[[:space:]]+(.*) ]]; then
            local header="${BASH_REMATCH[1]}"
            # Convert header to anchor (lowercase, replace spaces with dashes, remove special chars)
            local anchor=$(echo "$header" | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9 -]//g' | sed 's/ /-/g' | sed 's/--*/-/g')
            anchors+="$anchor"$'\n'
        fi
    done < "$file"

    echo "$anchors"
}

# Function to check if a file exists (handles relative paths)
check_file_exists() {
    local link="$1"
    local source_file="$2"
    local source_dir="$(dirname "$source_file")"

    # Remove anchor if present
    local file_path="${link%%#*}"

    # Skip if it's an anchor-only link
    if [[ -z "$file_path" ]]; then
        return 0
    fi

    # Handle absolute paths from project root
    if [[ "$file_path" == /* ]]; then
        file_path="$PROJECT_ROOT$file_path"
    # Handle relative paths
    else
        file_path="$source_dir/$file_path"
    fi

    # Normalize path
    file_path="$(cd "$source_dir" && realpath -m "$file_path" 2>/dev/null || echo "$file_path")"

    # Check if file exists
    if [[ -f "$file_path" ]]; then
        echo "$file_path"
        return 0
    else
        return 1
    fi
}

# Function to check anchor in a file
check_anchor() {
    local link="$1"
    local source_file="$2"

    # Extract anchor
    if [[ ! "$link" =~ \#(.+)$ ]]; then
        return 0  # No anchor to check
    fi

    local anchor="${BASH_REMATCH[1]}"
    local file_path="${link%%#*}"
    local source_dir="$(dirname "$source_file")"

    # If no file path, check anchor in current file
    if [[ -z "$file_path" ]]; then
        file_path="$source_file"
    else
        # Resolve file path
        if [[ "$file_path" == /* ]]; then
            file_path="$PROJECT_ROOT$file_path"
        else
            file_path="$source_dir/$file_path"
        fi
        file_path="$(cd "$source_dir" && realpath -m "$file_path" 2>/dev/null || echo "$file_path")"
    fi

    # Get anchors for the target file (cache them)
    if [[ -z "${file_anchors[$file_path]}" ]]; then
        if [[ -f "$file_path" ]]; then
            file_anchors[$file_path]=$(extract_anchors "$file_path")
        else
            file_anchors[$file_path]=""
        fi
    fi

    # Check if anchor exists
    if echo "${file_anchors[$file_path]}" | grep -qx "$anchor"; then
        return 0
    else
        return 1
    fi
}

# Function to validate links in a markdown file
validate_markdown_links() {
    local file="$1"
    local current_line=0

    while IFS= read -r line; do
        ((current_line++))

        # Extract markdown links: [text](url) or [text][ref]
        while [[ "$line" =~ \[([^\]]+)\]\(([^\)]+)\) ]]; do
            local link_text="${BASH_REMATCH[1]}"
            local link_url="${BASH_REMATCH[2]}"

            # Remove the matched portion to find next link
            line="${line#*\[${link_text}\]\(${link_url}\)}"

            # Skip external links (http, https, mailto, etc.)
            if [[ "$link_url" =~ ^(https?|mailto|ftp|tel): ]]; then
                continue
            fi

            ((total_links++))

            # Check if it's an anchor-only link
            if [[ "$link_url" =~ ^# ]]; then
                ((total_anchors++))
                if check_anchor "$link_url" "$file"; then
                    ((valid_anchors++))
                    ((valid_links++))
                else
                    ((broken_anchors++))
                    ((broken_links++))
                    errors+=("$file:$current_line: Broken anchor: $link_url")
                fi
            # Check file link
            else
                local target_file
                if target_file=$(check_file_exists "$link_url" "$file"); then
                    ((valid_links++))

                    # Check anchor if present
                    if [[ "$link_url" =~ \# ]]; then
                        ((total_anchors++))
                        if check_anchor "$link_url" "$file"; then
                            ((valid_anchors++))
                        else
                            ((broken_anchors++))
                            errors+=("$file:$current_line: Valid file but broken anchor: $link_url")
                        fi
                    fi
                else
                    ((broken_links++))
                    errors+=("$file:$current_line: Broken link: $link_url")
                fi
            fi
        done
    done < "$file"
}

# Find all markdown files in docs directory and README files
echo "Scanning for markdown files..."
file_count=0

# Process docs directory
if [[ -d "$PROJECT_ROOT/docs" ]]; then
    while IFS= read -r file; do
        ((file_count++))
        echo "  [$file_count] Processing: ${file#$PROJECT_ROOT/}"
        validate_markdown_links "$file"
    done < <(find "$PROJECT_ROOT/docs" -type f -name "*.md" 2>/dev/null || true)
fi

# Process main README
if [[ -f "$PROJECT_ROOT/README.md" ]]; then
    ((file_count++))
    echo "  [$file_count] Processing: README.md"
    validate_markdown_links "$PROJECT_ROOT/README.md"
fi

# Process SDK directory
if [[ -d "$PROJECT_ROOT/sdk" ]]; then
    while IFS= read -r file; do
        ((file_count++))
        echo "  [$file_count] Processing: ${file#$PROJECT_ROOT/}"
        validate_markdown_links "$file"
    done < <(find "$PROJECT_ROOT/sdk" -type f -name "*.md" 2>/dev/null | grep -v node_modules || true)
fi

echo ""
echo "Link Validation Results"
echo "======================="
echo "Total links checked: $total_links"
echo -e "${GREEN}Valid links: $valid_links${NC}"

if [[ $broken_links -gt 0 ]]; then
    echo -e "${RED}Broken links: $broken_links${NC}"
fi

if [[ $total_anchors -gt 0 ]]; then
    echo ""
    echo "Anchor Statistics:"
    echo "  Total anchors: $total_anchors"
    echo -e "  ${GREEN}Valid anchors: $valid_anchors${NC}"
    if [[ $broken_anchors -gt 0 ]]; then
        echo -e "  ${RED}Broken anchors: $broken_anchors${NC}"
    fi
fi

# Print errors
if [[ ${#errors[@]} -gt 0 ]]; then
    echo ""
    echo -e "${RED}Broken Links:${NC}"
    for error in "${errors[@]}"; do
        echo -e "${RED}  ✗ $error${NC}"
    done
fi

echo ""

# Exit with error if there are broken links
if [[ $broken_links -gt 0 ]]; then
    echo -e "${RED}Link validation FAILED${NC}"
    exit 1
else
    echo -e "${GREEN}Link validation PASSED${NC}"
    exit 0
fi
