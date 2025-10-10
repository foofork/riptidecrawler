//! Enhanced stealth features for crawl4ai parity
//!
//! This module contains enhancements to achieve feature parity with crawl4ai's
//! advanced anti-detection capabilities.

pub mod header_consistency;
pub mod screen_resolution;
pub mod timezone_enhanced;
pub mod webrtc_enhanced;

pub use header_consistency::HeaderConsistencyManager;
pub use screen_resolution::ScreenResolutionManager;
pub use timezone_enhanced::TimezoneManager;
pub use webrtc_enhanced::WebRtcEnhanced;
