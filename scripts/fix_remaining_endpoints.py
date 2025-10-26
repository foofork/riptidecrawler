#!/usr/bin/env python3
"""
Fix remaining POST/PUT/PATCH endpoints that don't have 415 status codes
"""

import re

def fix_openapi_file(filepath):
    with open(filepath, 'r') as f:
        lines = f.readlines()

    result = []
    i = 0
    modified_count = 0

    while i < len(lines):
        result.append(lines[i])

        # Check if this is a POST/PUT/PATCH operation
        if lines[i].strip() in ['post:', 'put:', 'patch:']:
            # Look ahead for responses section
            j = i + 1
            responses_start = None

            while j < len(lines) and j < i + 30:
                if 'responses:' in lines[j]:
                    responses_start = j
                    break
                j += 1

            if responses_start:
                # Check if this responses section has 415
                has_415 = False
                has_429 = False
                line_429 = None

                for k in range(responses_start, min(responses_start + 20, len(lines))):
                    if "'415':" in lines[k] or '"415":' in lines[k]:
                        has_415 = True
                    if "'429':" in lines[k] or '"429":' in lines[k]:
                        has_429 = True
                        line_429 = k
                    if 'operationId:' in lines[k]:
                        break

                # If has 429 but not 415, add the missing status codes
                if has_429 and not has_415 and line_429:
                    # Get indentation
                    indent = '        '

                    # Skip to the 429 line
                    while i < line_429:
                        i += 1
                        result.append(lines[i])

                    # Insert new status codes before 429
                    result.insert(-1, f"{indent}'400':\n")
                    result.insert(-1, f"{indent}  $ref: '#/components/responses/BadRequest'\n")
                    result.insert(-1, f"{indent}'415':\n")
                    result.insert(-1, f"{indent}  $ref: '#/components/responses/UnsupportedMediaType'\n")

                    # Look for the line after 429's reference (should be next non-comment line)
                    k = i + 1
                    while k < len(lines) and k < i + 5:
                        if 'RateLimitExceeded' in lines[k]:
                            i = k
                            result.append(lines[i])
                            # Add 503 after
                            result.append(f"{indent}'503':\n")
                            result.append(f"{indent}  $ref: '#/components/responses/ServiceUnavailable'\n")
                            modified_count += 1
                            break
                        k += 1

        i += 1

    with open(filepath, 'w') as f:
        f.writelines(result)

    return modified_count

if __name__ == '__main__':
    filepath = '/workspaces/eventmesh/docs/api/openapi.yaml'
    count = fix_openapi_file(filepath)
    print(f"✅ Fixed {count} additional endpoints")
