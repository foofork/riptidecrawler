#!/usr/bin/env python3
"""
Final OpenAPI fixes:
1. Add text/plain content to health/metrics 200 responses (preserving YAML structure)
2. Add 405 MethodNotAllowed component and references
"""

import re

def fix_openapi(filepath: str):
    with open(filepath, 'r') as f:
        lines = f.readlines()

    result = []
    i = 0
    text_plain_added = 0
    in_health_metrics = False
    current_operation = None

    while i < len(lines):
        line = lines[i]
        result.append(line)

        # Track if we're in a health or metrics operation
        if 'operationId:' in line:
            if any(x in line for x in ['health', 'metrics']):
                in_health_metrics = True
                current_operation = 'metrics' if 'metrics' in line else 'health'
            else:
                in_health_metrics = False
                current_operation = None

        # Look for 200 responses in health/metrics that need text/plain
        if in_health_metrics and line.strip() == "'200':":
            # Check next line for description
            if i + 1 < len(lines) and 'description:' in lines[i + 1]:
                result.append(lines[i + 1])  # description line
                i += 1

                # Check if content already exists
                if i + 1 < len(lines):
                    next_line = lines[i + 1]
                    if 'content:' not in next_line:
                        # Add content block
                        indent = '          '
                        if current_operation == 'metrics':
                            result.append(f'{indent}content:\n')
                            result.append(f'{indent}  text/plain:\n')
                            result.append(f'{indent}    schema:\n')
                            result.append(f'{indent}      type: string\n')
                            result.append(f'{indent}      example: "# HELP riptide_requests_total Total requests\\n# TYPE riptide_requests_total counter\\nriptide_requests_total 1250\\n"\n')
                        else:
                            result.append(f'{indent}content:\n')
                            result.append(f'{indent}  text/plain:\n')
                            result.append(f'{indent}    schema:\n')
                            result.append(f'{indent}      type: string\n')
                            result.append(f'{indent}      example: "OK"\n')
                        text_plain_added += 1

        i += 1

    # Write back
    with open(filepath, 'w') as f:
        f.writelines(result)

    return text_plain_added

def add_405_component(filepath: str):
    """Add 405 MethodNotAllowed response component"""
    with open(filepath, 'r') as f:
        content = f.read()

    if 'MethodNotAllowed:' in content:
        return False

    # Add after ServiceUnavailable component
    component = """
    MethodNotAllowed:
      description: Method Not Allowed - HTTP method not supported for this endpoint
      content:
        application/json:
          schema:
            $ref: '#/components/schemas/Error'
          example:
            error:
              message: "Method Not Allowed: This endpoint does not support the requested HTTP method"
              retryable: false
              status: 405
              type: method_not_allowed
"""

    # Insert before schemas section
    content = content.replace('\n  schemas:', component + '\n  schemas:')

    with open(filepath, 'w') as f:
        f.write(content)

    return True

def add_405_to_endpoints(filepath: str) -> int:
    """Add 405 references to GET endpoints (most common case)"""
    with open(filepath, 'r') as f:
        lines = f.readlines()

    result = []
    i = 0
    count = 0

    while i < len(lines):
        line = lines[i]
        result.append(line)

        # Look for GET endpoints with responses but no 405
        if line.strip() == 'get:':
            # Scan forward for responses section
            j = i + 1
            responses_line = None
            has_405 = False

            while j < len(lines) and j < i + 50:
                if lines[j].strip() == 'responses:':
                    responses_line = j
                if "'405':" in lines[j] or '"405":' in lines[j]:
                    has_405 = True
                    break
                if lines[j].strip() in ['post:', 'put:', 'patch:', 'delete:']:
                    break
                j += 1

            if responses_line and not has_405:
                # Copy lines up to just before operationId
                while i < len(lines):
                    current = lines[i]
                    result.append(current)
                    i += 1

                    # Look for operationId to insert 405 before it
                    if 'operationId:' in current:
                        # Insert 405 before operationId
                        indent = '        '
                        result.insert(-1, f"{indent}'405':\n")
                        result.insert(-1, f"{indent}  $ref: '#/components/responses/MethodNotAllowed'\n")
                        count += 1
                        break
                continue

        i += 1

    with open(filepath, 'w') as f:
        f.writelines(result)

    return count

if __name__ == '__main__':
    filepath = '/workspaces/eventmesh/docs/api/openapi.yaml'

    print("Step 1: Adding text/plain to health/metrics 200 responses...")
    text_plain_count = fix_openapi(filepath)
    print(f"✅ Added text/plain to {text_plain_count} responses\n")

    print("Step 2: Adding 405 MethodNotAllowed component...")
    added = add_405_component(filepath)
    print(f"✅ {'Added' if added else 'Already exists'} MethodNotAllowed component\n")

    print(f"🎉 OpenAPI fixes complete")
    print(f"   - {text_plain_count} text/plain content types added")
    print(f"   - 405 component {'added' if added else 'verified'}")
