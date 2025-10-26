#!/usr/bin/env python3
"""
Complete remaining OpenAPI fixes:
1. Add text/plain content to all health/metrics 200 responses
2. Add 405 Method Not Allowed responses to all endpoints
"""

import re
from typing import List, Tuple

def add_text_plain_to_200_responses(filepath: str) -> int:
    """Add text/plain content type to health/metrics 200 responses"""
    with open(filepath, 'r') as f:
        lines = f.readlines()

    result = []
    i = 0
    count = 0

    while i < len(lines):
        line = lines[i]
        result.append(line)

        # Look for health or metrics operationId
        if 'operationId:' in line and any(x in line for x in ['health', 'metrics']):
            operation_type = 'metrics' if 'metrics' in line else 'health'

            # Scan forward for 200 response without content
            j = i + 1
            while j < len(lines) and j < i + 30:
                if "'200':" in lines[j] or '"200":' in lines[j]:
                    # Check if next line has description
                    if j + 1 < len(lines) and 'description:' in lines[j + 1]:
                        # Add the 200 line and description
                        result.append(lines[j])
                        result.append(lines[j + 1])

                        # Check if there's already content on next line
                        if j + 2 < len(lines):
                            next_line = lines[j + 2]
                            if 'content:' not in next_line and ("'" in next_line or '$ref' in next_line or 'operationId' in next_line):
                                # No content, add it
                                indent = '          '
                                if operation_type == 'metrics':
                                    result.append(f'{indent}content:\n')
                                    result.append(f'{indent}  text/plain:\n')
                                    result.append(f'{indent}    schema:\n')
                                    result.append(f'{indent}      type: string\n')
                                    result.append(f'{indent}      example: "# HELP riptide_requests_total Total requests\\n# TYPE riptide_requests_total counter\\nriptide_requests_total{{status=\\"success\\"}} 1250\\n"\n')
                                else:
                                    result.append(f'{indent}content:\n')
                                    result.append(f'{indent}  text/plain:\n')
                                    result.append(f'{indent}    schema:\n')
                                    result.append(f'{indent}      type: string\n')
                                    result.append(f'{indent}      example: "OK"\n')
                                count += 1
                                i = j + 1  # Skip ahead
                                break
                    break
                j += 1

        i += 1

    with open(filepath, 'w') as f:
        f.writelines(result)

    return count

def add_405_responses(filepath: str) -> int:
    """Add 405 Method Not Allowed to all endpoints"""
    with open(filepath, 'r') as f:
        content = f.read()

    # First, add 405 response component if it doesn't exist
    if 'MethodNotAllowed:' not in content:
        # Find the responses section and add the new component
        response_component = """
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
        # Insert before the schemas section
        content = content.replace('\n  schemas:', response_component + '\n  schemas:')

    with open(filepath, 'w') as f:
        f.write(content)

    # Now add 405 references to endpoints
    with open(filepath, 'r') as f:
        lines = f.readlines()

    result = []
    i = 0
    count = 0

    while i < len(lines):
        line = lines[i]
        result.append(line)

        # Look for responses: section
        if line.strip() == 'responses:':
            # Check if this endpoint already has 405
            has_405 = False
            j = i + 1
            while j < len(lines) and j < i + 30:
                if 'responses:' in lines[j] or 'operationId:' in lines[j]:
                    break
                if "'405':" in lines[j] or '"405":' in lines[j]:
                    has_405 = True
                    break
                j += 1

            if not has_405:
                # Find the last status code and insert 405 after it
                last_response_line = i
                k = i + 1
                while k < len(lines) and k < i + 30:
                    if lines[k].strip().startswith("'") or lines[k].strip().startswith('"'):
                        if ':' in lines[k]:
                            last_response_line = k
                    if 'operationId:' in lines[k] or 'parameters:' in lines[k]:
                        break
                    k += 1

                # Find end of last response block
                m = last_response_line + 1
                while m < len(lines):
                    next_line = lines[m].strip()
                    if next_line.startswith("'") and ':' in next_line:
                        # Found next response code
                        break
                    if 'operationId:' in next_line:
                        break
                    m += 1

                # Insert 405 before operationId
                while i < m:
                    result.append(lines[i])
                    i += 1

                # Add 405 response
                result.append("        '405':\n")
                result.append("          $ref: '#/components/responses/MethodNotAllowed'\n")
                count += 1
                i -= 1  # Adjust since we'll increment at end of loop

        i += 1

    with open(filepath, 'w') as f:
        f.writelines(result)

    return count

if __name__ == '__main__':
    filepath = '/workspaces/eventmesh/docs/api/openapi.yaml'

    print("Adding text/plain content to 200 responses...")
    text_plain_count = add_text_plain_to_200_responses(filepath)
    print(f"✅ Added text/plain to {text_plain_count} responses")

    print("\nAdding 405 Method Not Allowed responses...")
    method_405_count = add_405_responses(filepath)
    print(f"✅ Added 405 responses to {method_405_count} endpoints")

    print(f"\n🎉 Total fixes: {text_plain_count + method_405_count}")
