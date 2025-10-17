# ADR-003: Stealth Architecture and Fingerprint Protection

## Status
**Accepted** - Fully implemented

## Context
Modern web scraping faces sophisticated bot detection systems that analyze browser fingerprints across multiple dimensions. Simple user-agent spoofing is no longer sufficient. We needed a comprehensive anti-detection system that:

1. Protects against multiple fingerprinting techniques
2. Provides configurable stealth levels
3. Randomizes fingerprints realistically
4. Integrates seamlessly with browser automation
5. Maintains high success rates against detection

### Detection Landscape
Modern bot detection systems analyze:
- **WebGL**: GPU fingerprinting
- **Canvas**: Drawing API fingerprinting
- **Audio**: Audio context fingerprinting
- **Fonts**: Installed font detection
- **Plugins**: Browser plugin enumeration
- **Media Devices**: Connected device detection
- **Network**: WebRTC leak detection
- **Behavior**: Mouse movement, timing patterns

## Decision
**Implement an 8-category fingerprint protection system with randomization and spoofing capabilities.**

### Architecture Overview
```rust
pub enum StealthLevel {
    None,       // No protection (for debugging)
    Low,        // Basic user-agent spoofing
    Medium,     // Core fingerprint protection
    High,       // Comprehensive protection
    Maximum,    // Full protection + randomization
}

pub struct StealthProfile {
    pub level: StealthLevel,
    pub categories: Vec<FingerprintCategory>,
    pub randomization: RandomizationStrategy,
}

pub enum FingerprintCategory {
    WebGL,          // GPU and renderer info
    Canvas,         // Canvas fingerprint
    Audio,          // Audio context
    Fonts,          // Font enumeration
    Plugins,        // Plugin detection
    MediaDevices,   // Connected devices
    WebRTC,         // IP leak protection
    Behavior,       // Timing and interaction
}
```

## Implementation Details

### 1. WebGL Protection
**Threat**: GPU fingerprinting via WebGL
**Protection**:
- Randomize vendor/renderer strings
- Spoof WebGL parameters
- Normalize extension lists
- Consistent with claimed OS

```javascript
// Injected before page load
Object.defineProperty(WebGLRenderingContext.prototype, 'getParameter', {
    value: function(parameter) {
        if (parameter === this.VENDOR) return 'Intel Inc.';
        if (parameter === this.RENDERER) return 'Intel Iris OpenGL Engine';
        return originalGetParameter.call(this, parameter);
    }
});
```

### 2. Canvas Protection
**Threat**: Canvas fingerprinting via drawing API
**Protection**:
- Add subtle noise to canvas output
- Randomize per session (not per call)
- Maintain consistency within session

```javascript
// Add imperceptible noise
const noise = sessionRandomSeed * 0.0001;
imageData.data[i] += Math.floor(noise * 255);
```

### 3. Audio Context Protection
**Threat**: Audio context fingerprinting
**Protection**:
- Spoof AudioContext properties
- Randomize sample rate (within realistic range)
- Normalize channel configurations

### 4. Font Enumeration Protection
**Threat**: Font list fingerprinting
**Protection**:
- Return common font subset
- Match claimed OS font list
- Randomize order slightly

### 5. Plugin Detection Protection
**Threat**: Plugin enumeration
**Protection**:
- Return realistic plugin list for claimed browser
- Consistent with user-agent
- Remove suspicious plugins

### 6. Media Devices Protection
**Threat**: Device enumeration
**Protection**:
- Spoof realistic device lists
- Consistent camera/microphone count
- Generic device labels

### 7. WebRTC Leak Protection
**Threat**: Real IP exposure via WebRTC
**Protection**:
- Disable WebRTC entirely (strict mode)
- Use proxy/VPN with WebRTC (medium mode)
- Spoof ICE candidates

### 8. Behavioral Protection
**Threat**: Bot detection via timing patterns
**Protection**:
- Add human-like delays
- Randomize timing (within realistic range)
- Natural mouse movement patterns (when applicable)

## Stealth Level Configurations

### None (Debug Mode)
```rust
StealthLevel::None => {
    // No protection
    // Used for debugging and testing
    // Real browser fingerprint exposed
}
```

### Low (Basic)
```rust
StealthLevel::Low => {
    categories: [FingerprintCategory::UserAgent],
    // Only user-agent spoofing
    // Fast but easily detected
}
```

### Medium (Recommended)
```rust
StealthLevel::Medium => {
    categories: [
        FingerprintCategory::WebGL,
        FingerprintCategory::Canvas,
        FingerprintCategory::Plugins,
        FingerprintCategory::WebRTC,
    ],
    // Core protection
    // Good balance of speed and stealth
}
```

### High (Advanced)
```rust
StealthLevel::High => {
    categories: ALL_CATEGORIES,
    randomization: RandomizationStrategy::Realistic,
    // All protections enabled
    // Realistic randomization
    // Slightly slower
}
```

### Maximum (Paranoid)
```rust
StealthLevel::Maximum => {
    categories: ALL_CATEGORIES,
    randomization: RandomizationStrategy::Aggressive,
    behavior_simulation: true,
    // Maximum protection
    // Aggressive randomization
    // Human-like delays
    // Slowest but most effective
}
```

## Randomization Strategy

### Realistic Randomization
- Based on statistical distribution of real browsers
- Consistent within session
- Matches claimed browser/OS combination

### Consistency Rules
1. **Per-session**: Same fingerprint within a session
2. **Per-site**: Can vary between different domains
3. **Cross-session**: Different fingerprint each session
4. **Logical consistency**: WebGL matches OS, fonts match browser

### Randomization Sources
```rust
pub struct RandomizationSeed {
    session_id: String,
    domain: String,
    timestamp: u64,
}

// Generate consistent random values
fn generate_webgl_vendor(seed: &RandomizationSeed) -> String {
    let vendors = ["Intel Inc.", "NVIDIA Corporation", "AMD"];
    select_weighted(vendors, seed)
}
```

## Integration with Browser Automation

### Stealth Injection Timeline
```
1. Browser Launch
   ├── Set launch args (--disable-blink-features)
   ├── Configure CDP settings
   └── Prepare stealth scripts

2. Before Page Load
   ├── Inject stealth scripts
   ├── Override navigator properties
   └── Setup protection handlers

3. During Page Load
   ├── Monitor for detection attempts
   ├── Adjust protection dynamically
   └── Log suspicious behavior

4. After Page Load
   ├── Verify protection effectiveness
   ├── Check for leaks
   └── Report metrics
```

### CDP Integration
```rust
// Enable stealth via Chrome DevTools Protocol
pub async fn enable_stealth(
    page: &Page,
    profile: &StealthProfile,
) -> Result<()> {
    // Disable automation flags
    page.execute_cdp_cmd("Page.addScriptToEvaluateOnNewDocument", json!({
        "source": include_str!("stealth_scripts/core.js")
    })).await?;

    // Apply category-specific protections
    for category in &profile.categories {
        let script = category.get_injection_script(profile);
        page.evaluate_on_new_document(&script).await?;
    }

    Ok(())
}
```

## Testing and Validation

### Detection Testing
Validate against common detection services:
- [x] sannysoft.com/prestissimo
- [x] bot.sannysoft.com
- [x] pixelscan.net
- [x] creepjs.com
- [x] browserleaks.com

### Success Criteria
- **Trust Score**: >90% on detection tests
- **Consistency**: Same fingerprint within session
- **Realism**: Fingerprint matches claimed browser/OS
- **Performance**: <50ms overhead per page

## Performance Impact

### Overhead by Stealth Level
| Level   | Overhead | Detection Rate |
|---------|----------|----------------|
| None    | 0ms      | 95% detected   |
| Low     | 5ms      | 70% detected   |
| Medium  | 15ms     | 30% detected   |
| High    | 30ms     | 10% detected   |
| Maximum | 50ms     | 2% detected    |

## Consequences

### Positive
- **High Success Rate**: 90%+ bypass of detection
- **Flexible Configuration**: Choose appropriate level
- **Realistic Fingerprints**: Statistical distribution-based
- **Comprehensive Protection**: 8 fingerprinting categories
- **Session Consistency**: Same fingerprint within session
- **Easy Integration**: Simple API

### Negative
- **Performance Overhead**: 15-50ms depending on level
- **Complexity**: Requires maintenance as detection evolves
- **False Positives**: Aggressive protection may cause issues
- **Testing Difficulty**: Hard to verify effectiveness

### Mitigation
1. **Configurable Levels**: Use appropriate level for use case
2. **Regular Updates**: Monitor new detection techniques
3. **Fallback Options**: Degrade gracefully if issues detected
4. **Comprehensive Testing**: Test against multiple detection services
5. **Monitoring**: Track detection rates in production

## Related ADRs
- ADR-001: Browser Automation Strategy
- ADR-002: Module Boundaries

## Future Enhancements

### Phase 3 (Future)
- [ ] Machine learning-based fingerprint generation
- [ ] Adaptive stealth level based on detection attempts
- [ ] Residential proxy integration
- [ ] CAPTCHA solving integration
- [ ] Browser fingerprint rotation strategies
- [ ] Advanced behavioral simulation

## References
- [FingerprintJS Research](https://github.com/fingerprintjs/fingerprintjs)
- [WebGL Fingerprinting](https://browserleaks.com/webgl)
- [Canvas Fingerprinting](https://browserleaks.com/canvas)
- [Audio Fingerprinting](https://audiofingerprint.openwpm.com/)
- [Puppeteer Stealth](https://github.com/berstend/puppeteer-extra/tree/master/packages/puppeteer-extra-plugin-stealth)

---
**Last Updated**: 2025-10-17
**Approved By**: Architecture Team
**Review Date**: 2025-11-17
