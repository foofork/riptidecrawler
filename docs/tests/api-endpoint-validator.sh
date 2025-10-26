#!/bin/bash
# API Endpoint Validator
# Compares API endpoints in documentation with OpenAPI spec

set -eo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Counters
total_endpoints_docs=0
total_endpoints_spec=0
matching_endpoints=0
undocumented_endpoints=0
invalid_endpoints=0

# Error collection
declare -a errors=()
declare -a warnings=()
declare -A spec_endpoints    # Endpoints from OpenAPI spec
declare -A doc_endpoints     # Endpoints from documentation

# Find project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Find OpenAPI spec
OPENAPI_SPEC=""
for spec_file in "$PROJECT_ROOT/docs/02-api-reference/openapi.yaml" \
                 "$PROJECT_ROOT/openapi.yaml" \
                 "$PROJECT_ROOT/spec/openapi.yaml" \
                 "$PROJECT_ROOT/api/openapi.yaml"; do
    if [[ -f "$spec_file" ]]; then
        OPENAPI_SPEC="$spec_file"
        break
    fi
done

echo "API Endpoint Validator"
echo "======================"
echo "Project Root: $PROJECT_ROOT"
echo ""

# Check if OpenAPI spec exists
if [[ -z "$OPENAPI_SPEC" ]]; then
    echo -e "${YELLOW}Warning: No OpenAPI specification found${NC}"
    echo "Searched for:"
    echo "  - docs/02-api-reference/openapi.yaml"
    echo "  - openapi.yaml"
    echo "  - spec/openapi.yaml"
    echo "  - api/openapi.yaml"
    echo ""
    echo "Skipping spec comparison, will only validate documentation format."
else
    echo "Found OpenAPI spec: $OPENAPI_SPEC"
    echo ""
fi

# Function to extract endpoints from OpenAPI spec
extract_spec_endpoints() {
    if [[ -z "$OPENAPI_SPEC" ]]; then
        return
    fi

    echo "Extracting endpoints from OpenAPI spec..."

    # Check if yq is available, otherwise use Python
    if command -v yq &> /dev/null; then
        # Use yq to extract paths
        while IFS= read -r line; do
            if [[ "$line" =~ ^([A-Z]+)[[:space:]]+(.+)$ ]]; then
                local method="${BASH_REMATCH[1]}"
                local path="${BASH_REMATCH[2]}"
                spec_endpoints["$method $path"]=1
                ((total_endpoints_spec++))
            fi
        done < <(yq eval '.paths | to_entries | .[] | .key as $path | .value | to_entries | .[] | (.key | upcase) + " " + $path' "$OPENAPI_SPEC" 2>/dev/null)
    else
        # Use Python as fallback
        while IFS= read -r line; do
            if [[ "$line" =~ ^([A-Z]+)[[:space:]]+(.+)$ ]]; then
                local method="${BASH_REMATCH[1]}"
                local path="${BASH_REMATCH[2]}"
                spec_endpoints["$method $path"]=1
                ((total_endpoints_spec++))
            fi
        done < <(python3 -c "
import yaml
import sys

try:
    with open('$OPENAPI_SPEC', 'r') as f:
        spec = yaml.safe_load(f)

    if 'paths' in spec:
        for path, methods in spec['paths'].items():
            for method in methods:
                if method.upper() in ['GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'HEAD', 'OPTIONS']:
                    print(f'{method.upper()} {path}')
except Exception as e:
    sys.stderr.write(f'Error parsing OpenAPI spec: {e}\n')
    sys.exit(1)
" 2>/dev/null)
    fi

    echo "  Found $total_endpoints_spec endpoints in spec"
}

# Function to extract endpoints from documentation
extract_doc_endpoints() {
    local file="$1"
    local current_line=0

    while IFS= read -r line; do
        ((current_line++))

        # Match API endpoint patterns:
        # GET /api/v1/endpoint
        # POST /endpoint
        # `GET /api/endpoint`
        # **GET** /endpoint

        # Pattern 1: Direct HTTP method + path
        if [[ "$line" =~ (GET|POST|PUT|DELETE|PATCH|HEAD|OPTIONS)[[:space:]]+(/[^[:space:]]*) ]]; then
            local method="${BASH_REMATCH[1]}"
            local path="${BASH_REMATCH[2]}"
            # Clean up path (remove trailing punctuation)
            path="${path%%[,;.]}"
            doc_endpoints["$method $path $file:$current_line"]=1
            ((total_endpoints_docs++))
        fi

        # Pattern 2: In code blocks (curl commands)
        if [[ "$line" =~ curl.*-X[[:space:]]*(GET|POST|PUT|DELETE|PATCH)[[:space:]].*http[s]?://[^/]+(/.+) ]]; then
            local method="${BASH_REMATCH[1]}"
            local path="${BASH_REMATCH[2]}"
            # Clean path
            path="${path%% *}"
            path="${path%%[\"\']*}"
            doc_endpoints["$method $path $file:$current_line"]=1
            ((total_endpoints_docs++))
        fi

        # Pattern 3: curl without -X (defaults to GET)
        if [[ "$line" =~ curl[^-]*http[s]?://[^/]+(/.+) ]] && [[ ! "$line" =~ -X ]]; then
            local path="${BASH_REMATCH[1]}"
            path="${path%% *}"
            path="${path%%[\"\']*}"
            doc_endpoints["GET $path $file:$current_line"]=1
            ((total_endpoints_docs++))
        fi
    done < "$file"
}

# Extract endpoints from spec
extract_spec_endpoints

# Extract endpoints from documentation
echo ""
echo "Extracting endpoints from documentation..."
file_count=0

# Process docs directory
if [[ -d "$PROJECT_ROOT/docs" ]]; then
    while IFS= read -r file; do
        ((file_count++))
        extract_doc_endpoints "$file"
    done < <(find "$PROJECT_ROOT/docs" -type f -name "*.md" 2>/dev/null || true)
fi

# Process main README
if [[ -f "$PROJECT_ROOT/README.md" ]]; then
    ((file_count++))
    extract_doc_endpoints "$PROJECT_ROOT/README.md"
fi

# Process SDK directory
if [[ -d "$PROJECT_ROOT/sdk" ]]; then
    while IFS= read -r file; do
        ((file_count++))
        extract_doc_endpoints "$file"
    done < <(find "$PROJECT_ROOT/sdk" -type f -name "*.md" 2>/dev/null | grep -v node_modules || true)
fi

echo "  Found $total_endpoints_docs endpoint references in docs"
echo ""

# Compare endpoints
if [[ $total_endpoints_spec -gt 0 ]]; then
    echo "Validating documented endpoints against spec..."

    for doc_endpoint in "${!doc_endpoints[@]}"; do
        # Extract method and path (ignore file:line)
        if [[ "$doc_endpoint" =~ ^([A-Z]+)[[:space:]]+([^[:space:]]+)[[:space:]]+(.+)$ ]]; then
            local method="${BASH_REMATCH[1]}"
            local path="${BASH_REMATCH[2]}"
            local location="${BASH_REMATCH[3]}"
            local key="$method $path"

            # Check if endpoint exists in spec
            if [[ -n "${spec_endpoints[$key]}" ]]; then
                ((matching_endpoints++))
            else
                # Check if it's a parameterized path (e.g., /users/123 vs /users/{id})
                local found=0
                for spec_key in "${!spec_endpoints[@]}"; do
                    if [[ "$spec_key" =~ ^$method[[:space:]] ]]; then
                        local spec_path="${spec_key#$method }"
                        # Convert {param} to regex
                        local pattern=$(echo "$spec_path" | sed 's/{[^}]*}/[^\/]+/g')
                        if [[ "$path" =~ ^$pattern$ ]]; then
                            ((matching_endpoints++))
                            found=1
                            break
                        fi
                    fi
                done

                if [[ $found -eq 0 ]]; then
                    ((invalid_endpoints++))
                    errors+=("$location: Endpoint not in spec: $method $path")
                fi
            fi
        fi
    done

    # Check for undocumented endpoints in spec
    echo "Checking for undocumented endpoints..."
    for spec_key in "${!spec_endpoints[@]}"; do
        local found=0
        for doc_endpoint in "${!doc_endpoints[@]}"; do
            if [[ "$doc_endpoint" =~ ^$spec_key[[:space:]] ]]; then
                found=1
                break
            fi
        done

        if [[ $found -eq 0 ]]; then
            ((undocumented_endpoints++))
            warnings+=("Endpoint in spec but not documented: $spec_key")
        fi
    done
fi

# Print results
echo ""
echo "Validation Results"
echo "=================="
echo "Documentation:"
echo "  Total endpoint references: $total_endpoints_docs"

if [[ $total_endpoints_spec -gt 0 ]]; then
    echo ""
    echo "OpenAPI Spec:"
    echo "  Total endpoints: $total_endpoints_spec"
    echo ""
    echo "Comparison:"
    echo -e "  ${GREEN}Matching endpoints: $matching_endpoints${NC}"

    if [[ $invalid_endpoints -gt 0 ]]; then
        echo -e "  ${RED}Invalid/outdated endpoints: $invalid_endpoints${NC}"
    fi

    if [[ $undocumented_endpoints -gt 0 ]]; then
        echo -e "  ${YELLOW}Undocumented endpoints: $undocumented_endpoints${NC}"
    fi
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
if [[ ${#warnings[@]} -gt 0 ]]; then
    echo ""
    echo -e "${YELLOW}Warnings:${NC}"
    for warning in "${warnings[@]}"; do
        echo -e "${YELLOW}  ⚠ $warning${NC}"
    done
fi

echo ""

# Exit with error if there are invalid endpoints
if [[ $invalid_endpoints -gt 0 ]]; then
    echo -e "${RED}Validation FAILED${NC}"
    exit 1
else
    echo -e "${GREEN}Validation PASSED${NC}"
    exit 0
fi
