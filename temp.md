

 ✅  Loaded specification from docs/api/openapi.yaml (in 0.29s)                 

     Base URL:         http://localhost:8080                                    
     Specification:    Open API 3.0.0                                           
     Operations:       100 selected / 100 total                                 


 ✅  API capabilities:                                                          

     Supports NULL byte in headers:    ✘                                        

 ❌  Examples (in 1.97s)                                                        
                                                                                
     ✅   9 passed  ❌  19 failed  ⏭   72 skipped                               

 ❌  Coverage (in 3.53s)                                                        
                                                                                
     ✅  38 passed  ❌  62 failed                                               

 ❌  Fuzzing (in 1.51s)                                                         
                                                                                
     ✅   1 passed  ❌  99 failed                                               

=================================== FAILURES ===================================
________________________ GET /api/v1/tables/{id}/export ________________________
1. Test Case ID: mwztjn

- Response violates schema

    'message' is a required property

    Schema:

        {
            "required": [
                "message",
                "retryable",
                "status",
                "type"
            ],
            "type": "object",
            "properties": {
                "message": {

    Valid data should have been accepted
    Expected: 2xx, 401, 403, 404, 5xx

[429] Too Many Requests:

    `{"error":"RateLimitExceeded","message":"Rate limit exceeded: Resource limit exceeded: Global rate limit exceeded","retry_after_seconds":60}`

Reproduce with: 

    curl -X DELETE http://localhost:8080/api/v1/profiles/clear

=================================== WARNINGS ===================================

Missing test data: 1 operation repeatedly returned 404 Not Found, preventing tests from reaching your API's core logic

  - GET /api/v1/profiles/{domain}/stats

💡 Provide realistic parameter values in your config file so tests can access existing resources

=================================== SUMMARY ====================================

API Operations:
  Selected: 100/100
  Tested: 100

Test Phases:
  ❌ Examples
  ❌ Coverage
  ❌ Fuzzing
  ⏭  Stateful (not applicable)

Failures:
  ❌ Server error: 7
  ❌ Response violates schema: 2
  ❌ API accepted schema-violating request: 1
  ❌ API rejected schema-compliant request: 128
  ❌ Undocumented Content-Type: 1
  ❌ Undocumented HTTP status code: 142
  ❌ Unsupported method incorrect response: 15

Warnings:
  ⚠️ Missing valid test data: 1 operation repeatedly returned 404 responses

Test cases:
  729 generated, 167 found 296 unique failures

Seed: 104913228381974101414082996839710455797

============================ 296 failures in 7.05s =============================
Error: Process completed with exit code 1.