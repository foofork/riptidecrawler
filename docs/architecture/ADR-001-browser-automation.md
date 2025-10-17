# ADR-001: Browser Automation Strategy

## Status
**Accepted** - Migration in progress (Phase 1 & 2)

## Context
The EventMesh/RipTide project requires robust browser automation for web scraping, content extraction, and stealth operations. We needed to choose between multiple automation libraries with different capabilities and performance characteristics.

### Options Evaluated
1. **chromiumoxide** - Current implementation
   - Pure Rust Chrome DevTools Protocol (CDP) implementation
   - Direct Chrome/Chromium control
   - Limited concurrency and performance

2. **spider-chrome** - Target implementation
   - High-performance Chrome automation
   - Built-in concurrency support (200+ concurrent sessions)
   - Advanced CDP features
   - Active maintenance and community

3. **headless-chrome** - Alternative
   - Simpler API but less maintained
   - Limited feature set

## Decision
**Migrate from chromiumoxide to spider-chrome** as the primary browser automation engine.

### Rationale
1. **Performance**: spider-chrome offers 2-3x better performance with native async/await support
2. **Concurrency**: Built-in support for 200+ concurrent browser sessions vs 50-70 with chromiumoxide
3. **Features**: More comprehensive CDP implementation including advanced stealth features
4. **Maintenance**: Active development and community support
5. **Integration**: Better integration with async Rust ecosystem

### Migration Strategy
Phase-based migration with hybrid architecture:
- **Phase 1**: Introduce spider-chrome alongside chromiumoxide
- **Phase 2**: Implement facade pattern for seamless switching
- **Phase 3**: Migrate core functionality to spider-chrome
- **Phase 4**: Remove chromiumoxide dependency

## Implementation Details

### Architecture Pattern
**Engine Facade Pattern**:
```rust
pub trait BrowserEngine {
    async fn navigate(&self, url: &str) -> Result<()>;
    async fn get_content(&self) -> Result<String>;
    async fn screenshot(&self) -> Result<Vec<u8>>;
}

pub enum EngineType {
    Spider,      // Default for new operations
    Chromium,    // Fallback for compatibility
    Auto,        // Automatic selection based on requirements
}
```

### Performance Targets
- **Concurrency**: 200+ concurrent sessions (vs 50-70 current)
- **Response Time**: <100ms for simple operations (vs 150-300ms)
- **Memory**: <50MB per session (vs 80-120MB)
- **Reliability**: 99.9% operation success rate

### Stealth Integration
spider-chrome provides better foundation for stealth operations:
- Advanced fingerprint randomization
- Better CDP event handling
- More reliable JavaScript execution
- Enhanced cookie and storage management

## Consequences

### Positive
- **+200% concurrency improvement**
- Better CDP protocol support
- Improved stealth capabilities
- Active community and maintenance
- Better async/await integration
- More comprehensive error handling

### Negative
- Migration effort required (~3-4 weeks)
- Potential breaking changes in automation code
- Need to maintain dual engine support temporarily
- Learning curve for spider-chrome API differences

### Mitigation
1. **Facade Pattern**: Abstract engine differences behind common interface
2. **Gradual Migration**: Phase-based approach allows for testing and validation
3. **Fallback Support**: Keep chromiumoxide as fallback during migration
4. **Comprehensive Testing**: Maintain test coverage during migration
5. **Documentation**: Update all automation-related docs

## Related ADRs
- ADR-003: Stealth Architecture
- ADR-002: Module Boundaries

## References
- [spider-chrome GitHub](https://github.com/spider-rs/spider-chrome)
- [chromiumoxide GitHub](https://github.com/mattsse/chromiumoxide)
- [Chrome DevTools Protocol](https://chromedevtools.github.io/devtools-protocol/)

## Timeline
- **Week 1-2**: Facade pattern implementation
- **Week 3-4**: Core migration (render, extraction)
- **Week 5-6**: Stealth features migration
- **Week 7-8**: Testing and optimization

## Success Metrics
- ✅ 200+ concurrent sessions supported
- ✅ <100ms response time for basic operations
- ✅ 99.9% reliability across all operations
- ✅ Zero breaking changes to public API
- ✅ All tests passing with new engine

---
**Last Updated**: 2025-10-17
**Approved By**: Architecture Team
**Review Date**: 2025-11-17
