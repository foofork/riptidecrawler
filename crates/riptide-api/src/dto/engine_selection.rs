//! Engine Selection API DTOs - Request/Response types for engine selection
//!
//! Extracted from handlers/engine_selection.rs (Phase 3 Sprint 3.1)
//! Contains all 4 DTOs + conversion traits

use riptide_facade::facades::EngineSelectionCriteria;
use riptide_reliability::engine_selection::EngineSelectionFlags;
use serde::Deserialize;

/// Request for engine analysis
#[derive(Debug, Deserialize)]
pub struct AnalyzeRequest {
    pub html: String,
    pub url: String,
}

impl AnalyzeRequest {
    pub fn to_criteria(&self) -> EngineSelectionCriteria {
        EngineSelectionCriteria {
            html: self.html.clone(),
            url: self.url.clone(),
            flags: EngineSelectionFlags::default(),
        }
    }
}

/// Request for engine decision with flags
#[derive(Debug, Deserialize)]
pub struct DecideRequest {
    pub html: String,
    pub url: String,
    #[serde(default)]
    pub flags: EngineSelectionFlagsRequest,
}

impl DecideRequest {
    pub fn to_criteria(&self) -> EngineSelectionCriteria {
        EngineSelectionCriteria {
            html: self.html.clone(),
            url: self.url.clone(),
            flags: self.flags.clone().into(),
        }
    }
}

/// Engine selection feature flags (API request format)
#[derive(Debug, Clone, Deserialize, Default)]
pub struct EngineSelectionFlagsRequest {
    #[serde(default)]
    pub use_visible_text_density: bool,
    #[serde(default)]
    pub detect_placeholders: bool,
    #[serde(default)]
    pub probe_first_spa: bool,
}

impl From<EngineSelectionFlagsRequest> for EngineSelectionFlags {
    fn from(req: EngineSelectionFlagsRequest) -> Self {
        Self {
            use_visible_text_density: req.use_visible_text_density,
            detect_placeholders: req.detect_placeholders,
            probe_first_spa: req.probe_first_spa,
        }
    }
}

/// Request to toggle probe-first mode
#[derive(Debug, Deserialize)]
pub struct ProbeFirstRequest {
    pub enabled: bool,
}
