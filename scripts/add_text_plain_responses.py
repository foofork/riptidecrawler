#!/usr/bin/env python3
"""
Add text/plain content type to health and metrics endpoints
"""

import re

def add_text_plain_to_health_endpoints(filepath):
    with open(filepath, 'r') as f:
        content = f.read()

    # Pattern for health check 200 responses without content
    health_200_pattern = r"(operationId:\s+health_check[^\n]*\n(?:.*\n)*?)\s+'200':\s*\n\s+description:\s+([^\n]+)\n(?!\s+content:)"

    def replace_health_200(match):
        prefix = match.group(1)
        description = match.group(2)
        return f"{prefix}        '200':\n          description: {description}\n          content:\n            text/plain:\n              schema:\n                type: string\n                example: \"OK\"\n"

    content = re.sub(health_200_pattern, replace_health_200, content)

    # Pattern for health check 503 responses without content
    health_503_pattern = r"(\s+'503':\s*\n\s+description:\s+Service unhealthy)\n(?!\s+content:)"

    def replace_health_503(match):
        return f"{match.group(1)}\n          content:\n            text/plain:\n              schema:\n                type: string\n                example: \"Service Unavailable\"\n"

    content = re.sub(health_503_pattern, replace_health_503, content)

    # Pattern for metrics endpoints 200 responses without content
    metrics_200_pattern = r"(operationId:\s+metrics_prometheus[^\n]*\n(?:.*\n)*?)\s+'200':\s*\n\s+description:\s+([^\n]+)\n(?!\s+content:)"

    def replace_metrics_200(match):
        prefix = match.group(1)
        description = match.group(2)
        return f"{prefix}        '200':\n          description: {description}\n          content:\n            text/plain:\n              schema:\n                type: string\n                example: \"# HELP riptide_requests_total Total number of requests\\n# TYPE riptide_requests_total counter\\nriptide_requests_total 100\\n\"\n"

    content = re.sub(metrics_200_pattern, replace_metrics_200, content)

    with open(filepath, 'w') as f:
        f.write(content)

    return content.count('text/plain')

if __name__ == '__main__':
    filepath = '/workspaces/eventmesh/docs/api/openapi.yaml'
    count = add_text_plain_to_health_endpoints(filepath)
    print(f"✅ Added text/plain to {count} responses")
