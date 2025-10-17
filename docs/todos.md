✅ Proper Architecture: CLI as API Client

  CLI ──[HTTP]──> API Server ──> riptide-headless
                                riptide-stealth
                                chromiumoxide

  Benefits of CLI → API Approach

  1. Single Source of Truth
    - All business logic in one place
    - Consistent behavior across interfaces
    - One codebase to maintain
  2. Better Resource Management
    - API server manages browser pool (3 max)
    - Rate limiting enforced centrally (1.5 RPS)
    - Memory limits controlled globally (2GB)
    - No duplicate browser instances
  3. Centralized Auth & Security
    - API key authentication
    - Audit logging
    - Request tracking
    - Usage quotas
  4. Easier Testing & Updates
    - Test API once, works for CLI/web/mobile
    - Deploy API update, all clients benefit
    - No need to rebuild/redistribute CLI
  5. Multi-User Support
    - Multiple users share same browser pool
    - Better resource utilization
    - Centralized monitoring

  The .env Shows This Was Intended

  Your .env.example already has:
  # CLI is supposed to use API!
  RIPTIDE_API_URL=http://localhost:8080
  RIPTIDE_API_KEY=your_api_key_here

  🔧 Recommended Architecture: Hybrid Approach

  // CLI should try API first, fallback to direct
  pub async fn execute_render(args: RenderArgs) -> Result<()> {
      // Try API first
      if let Ok(api_url) = std::env::var("RIPTIDE_API_URL") {
          if let Ok(api_key) = std::env::var("RIPTIDE_API_KEY") {
              match execute_via_api(&api_url, &api_key, args).await {
                  Ok(result) => return Ok(result),
                  Err(e) => {
                      warn!("API call failed: {}, falling back to direct mode", e);
                  }
              }
          }
      }

      // Fallback to direct implementation
      execute_direct(args).await
  }

  When Direct CLI Implementation Makes Sense

  Keep direct implementation as fallback for:
  - Development/testing without API server
  - Offline usage
  - Single-user/developer tools
  - When API is unavailable
  - Air-gapped environments

  📊 Current Issues in Your Codebase

  Looking at render.rs:434-436, I see:
  // Screenshot functionality temporarily disabled - type visibility issues

  This is exactly the problem - you're trying to maintain two parallel implementations and
  running into issues!

  🎯 Action Items

  1. Make CLI an API client by default
    - Use reqwest to call API endpoints
    - Remove duplicate browser management code
    - Keep thin wrapper for CLI UX
  2. Keep Direct Mode as Fallback
    - Useful for development
    - Document as "advanced/offline mode"
    - Require explicit flag: --direct-mode
  3. Standardize on API Contract
    - CLI and API return same JSON structures
    - Consistent error codes
    - Unified output formats
  4. Configuration Priority
  1. RIPTIDE_API_URL (prefer API)
  2. Fallback to direct mode if not set
  3. --direct flag to force direct mode

  💡 Example: How CLI Should Work

  # Uses API by default (recommended)
  export RIPTIDE_API_URL=http://localhost:8080
  export RIPTIDE_API_KEY=your_key
  riptide-cli render --url https://wired.com --screenshot full

  # Direct mode (fallback/development)
  riptide-cli render --url https://wired.com --screenshot full --direct

  # Or unset API env vars
  unset RIPTIDE_API_URL
  riptide-cli render --url https://wired.com --screenshot full

  Summary: Yes, for a proper production system, the CLI should primarily be an API client.
  The current duplication is a maintenance burden and architectural smell. Keep direct mode
  as a fallback, but encourage API usage as the primary path.

> ok.  let's spin up analysts and architect to properly address the systatic issues related 
to this, provide a plan to address all of the areas to correctly implement the fixes, 
standardize the env variables so people can store things where they want for all of their 
extractions and test artifacts and reports and anything else missing.  spin up a team to 
implement it using the hive-mind 



### Minor Issues (Non-Blocking):
1. **PDF Memory Benchmark Test:** 1 test failing (unrelated to extraction functionality)
2. **Dead Code Warnings:** 41 warnings for unused functions (can be cleaned up separately)
3. **Render Module:** Screenshot and PDF functionality temporarily disabled (chromiumoxide type visibility)

### Recommendations:
1. Run full integration test suite after release build completes
2. Address dead code warnings in future cleanup sprint
3. Re-enable render module screenshot/PDF features after resolving type visibility
4. Consider adding more health endpoint integration tests
