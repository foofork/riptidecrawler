#!/usr/bin/env python3
"""
Script to add missing status code responses to POST/PUT/PATCH endpoints in openapi.yaml
"""

import re
import sys

def add_status_codes_to_endpoint(endpoint_text, has_request_body=True):
    """Add missing status codes to an endpoint's responses section"""

    # Check if already has the status codes we want to add
    has_400 = "'400':" in endpoint_text or '"400":' in endpoint_text
    has_415 = "'415':" in endpoint_text or '"415":' in endpoint_text
    has_503 = "'503':" in endpoint_text or '"503":' in endpoint_text

    if has_400 and has_415 and has_503:
        return endpoint_text  # Already has all status codes

    # Find the responses section
    responses_match = re.search(r'(\s+)(responses:.*?)(\n\s+\w+:|$)', endpoint_text, re.DOTALL)
    if not responses_match:
        return endpoint_text

    indent = responses_match.group(1)
    responses_section = responses_match.group(2)

    # Find the last response code to insert before operationId or next section
    lines = responses_section.split('\n')
    result_lines = []
    inserted = False

    for i, line in enumerate(lines):
        result_lines.append(line)

        # After 429 response, add our new status codes
        if "'429':" in line or '"429":' in line:
            # Check if this is the last response before operationId
            next_non_empty = None
            for j in range(i+1, len(lines)):
                if lines[j].strip():
                    next_non_empty = lines[j]
                    break

            if next_non_empty and ('operationId:' in next_non_empty or not next_non_empty.startswith(indent + '  ')):
                if not inserted:
                    # Add 400, 415, 503 before operationId
                    if not has_400:
                        result_lines.insert(-1, f"{indent}  '400':")
                        result_lines.insert(-1, f"{indent}    $ref: '#/components/responses/BadRequest'")
                    if not has_415:
                        result_lines.insert(-1, f"{indent}  '415':")
                        result_lines.insert(-1, f"{indent}    $ref: '#/components/responses/UnsupportedMediaType'")
                    # Re-add 429
                    result_lines.append(line)
                    if not has_503:
                        result_lines.append(f"{indent}  '503':")
                        result_lines.append(f"{indent}    $ref: '#/components/responses/ServiceUnavailable'")
                    inserted = True
                    result_lines.pop(result_lines.index(line, -10))  # Remove duplicate 429

    return '\n'.join(result_lines)


def process_openapi_file(filepath):
    """Process the OpenAPI YAML file and add missing status codes"""

    with open(filepath, 'r') as f:
        content = f.read()

    # First, add the reusable component responses
    components_responses_pattern = r'(    RateLimitExceeded:.*?retry_after_seconds: 60\n)'

    new_responses = """    RateLimitExceeded:
      description: Rate Limit Exceeded - Too many requests
      content:
        application/json:
          schema:
            $ref: '#/components/schemas/RateLimitError'
          example:
            error: RateLimitExceeded
            message: "Rate limit exceeded: Resource limit exceeded: Global rate limit exceeded"
            retry_after_seconds: 60

    UnsupportedMediaType:
      description: Unsupported Media Type - Content-Type header missing or unsupported
      content:
        application/json:
          schema:
            $ref: '#/components/schemas/Error'
          example:
            error:
              message: "Expected request with `Content-Type: application/json`"
              retryable: false
              status: 415
              type: "unsupported_media_type"

    ServiceUnavailable:
      description: Service Unavailable - Dependency unavailable
      content:
        application/json:
          schema:
            $ref: '#/components/schemas/Error'
          example:
            error:
              message: "Dependency unavailable: redis - Connection timeout"
              retryable: true
              status: 503
              type: "dependency_error"
"""

    if 'UnsupportedMediaType:' not in content:
        content = re.sub(components_responses_pattern, new_responses + '\n', content, flags=re.DOTALL)

    # List of POST/PUT/PATCH endpoints that need status codes
    endpoints_needing_codes = [
        '/crawl:',
        '/api/v1/crawl:',
        '/crawl/stream:',
        '/crawl/sse:',
        '/deepsearch:',
        '/deepsearch/stream:',
        '/render:',
        '/api/v1/render:',
        '/extract:',
        '/api/v1/extract:',
        '/spider/crawl:',
        '/spider/status:',
        '/spider/control:',
        '/strategies/crawl:',
        '/pdf/process:',
        '/pdf/process-stream:',
        '/stealth/configure:',
        '/stealth/test:',
        '/api/v1/tables/extract:',
        '/api/v1/llm/providers/switch:',
        '/api/v1/llm/config:',
        '/sessions:',
        '/sessions/cleanup:',
        '/sessions/{session_id}/extend:',
        '/sessions/{session_id}/cookies:',
        '/workers/jobs:',
        '/workers/schedule:',
        '/api/v1/browser/session:',
        '/api/v1/browser/action:',
        '/admin/tenants:',
        '/admin/tenants/{id}:',
        '/admin/cache/warm:',
        '/admin/state/reload:',
        '/api/v1/profiles:',
        '/api/v1/profiles/{domain}:',
        '/api/v1/profiles/batch:',
        '/api/v1/profiles/{domain}/warm:',
        '/api/v1/engine/analyze:',
        '/api/v1/engine/decide:',
        '/api/v1/engine/probe-first:',
    ]

    # Process each endpoint - find POST/PUT/PATCH operations and add status codes
    for endpoint in endpoints_needing_codes:
        # Find the endpoint section
        pattern = rf'(  {re.escape(endpoint)}\n    (post|put|patch):.*?)(  /\w+|components:)'
        matches = list(re.finditer(pattern, content, re.DOTALL))

        for match in matches:
            endpoint_section = match.group(1)

            # Add status codes if responses section exists and doesn't have 415
            if 'responses:' in endpoint_section and '415' not in endpoint_section:
                # Find where to insert - after 200/201 but before 429
                modified_section = endpoint_section

                # Simple approach: add after last existing response code
                if "'429':" in modified_section:
                    # Insert 400, 415, 503 around 429
                    modified_section = modified_section.replace(
                        "        '429':",
                        "        '400':\n          $ref: '#/components/responses/BadRequest'\n" +
                        "        '415':\n          $ref: '#/components/responses/UnsupportedMediaType'\n" +
                        "        '429':"
                    )
                    modified_section = modified_section.replace(
                        "          $ref: '#/components/responses/RateLimitExceeded'\n      operationId:",
                        "          $ref: '#/components/responses/RateLimitExceeded'\n" +
                        "        '503':\n          $ref: '#/components/responses/ServiceUnavailable'\n" +
                        "      operationId:"
                    )

                    content = content.replace(endpoint_section, modified_section)

    return content


if __name__ == '__main__':
    filepath = '/workspaces/eventmesh/docs/api/openapi.yaml'

    try:
        result = process_openapi_file(filepath)

        # Write the result
        with open(filepath, 'w') as f:
            f.write(result)

        print(f"✅ Successfully updated {filepath}")
        print(f"   - Added UnsupportedMediaType and ServiceUnavailable components")
        print(f"   - Added 400, 415, 503 status codes to POST/PUT/PATCH endpoints")

    except Exception as e:
        print(f"❌ Error processing file: {e}", file=sys.stderr)
        sys.exit(1)
