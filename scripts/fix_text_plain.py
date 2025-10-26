#!/usr/bin/env python3
"""
Add text/plain content type to health and metrics endpoint responses
"""

def fix_openapi_file(filepath):
    with open(filepath, 'r') as f:
        lines = f.readlines()

    result = []
    i = 0
    modified_count = 0
    in_health_or_metrics = False
    current_operation = None

    while i < len(lines):
        line = lines[i]
        result.append(line)

        # Track if we're in a health or metrics endpoint
        if 'operationId:' in line:
            if any(x in line for x in ['health_check', 'metrics_prometheus', 'health_detailed', 'health_component', 'health_metrics']):
                in_health_or_metrics = True
                current_operation = line.strip()
            else:
                in_health_or_metrics = False
                current_operation = None

        # Look for 200 responses without content in health/metrics endpoints
        if in_health_or_metrics and ("'200':" in line or '"200":' in line):
            # Check next line for description
            if i + 1 < len(lines) and 'description:' in lines[i + 1]:
                result.append(lines[i + 1])  # description line
                i += 1

                # Check if next line already has 'content:' or is another response code
                if i + 1 < len(lines):
                    next_line = lines[i + 1]
                    if 'content:' not in next_line and ("'" not in next_line or 'description' in next_line):
                        # Add content block
                        indent = '          '
                        if 'metrics' in current_operation:
                            result.append(f'{indent}content:\n')
                            result.append(f'{indent}  text/plain:\n')
                            result.append(f'{indent}    schema:\n')
                            result.append(f'{indent}      type: string\n')
                            result.append(f'{indent}      example: "# HELP riptide_requests_total Total requests\\n# TYPE riptide_requests_total counter\\nriptide_requests_total 100\\n"\n')
                        else:  # health endpoints
                            result.append(f'{indent}content:\n')
                            result.append(f'{indent}  text/plain:\n')
                            result.append(f'{indent}    schema:\n')
                            result.append(f'{indent}      type: string\n')
                            result.append(f'{indent}      example: "OK"\n')
                        modified_count += 1

        # Look for 503 responses without content in health/metrics endpoints
        if in_health_or_metrics and ("'503':" in line or '"503":' in line):
            # Check next line for description
            if i + 1 < len(lines) and 'description:' in lines[i + 1]:
                result.append(lines[i + 1])  # description line
                i += 1

                # Check if next line already has 'content:' or is another key
                if i + 1 < len(lines):
                    next_line = lines[i + 1]
                    if 'content:' not in next_line and '$ref' not in next_line:
                        # Add content block
                        indent = '          '
                        result.append(f'{indent}content:\n')
                        result.append(f'{indent}  text/plain:\n')
                        result.append(f'{indent}    schema:\n')
                        result.append(f'{indent}      type: string\n')
                        result.append(f'{indent}      example: "Service Unavailable"\n')
                        modified_count += 1

        i += 1

    with open(filepath, 'w') as f:
        f.writelines(result)

    return modified_count

if __name__ == '__main__':
    filepath = '/workspaces/eventmesh/docs/api/openapi.yaml'
    count = fix_openapi_file(filepath)
    print(f"✅ Added text/plain content to {count} responses")
