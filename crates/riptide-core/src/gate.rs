use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateFeatures {
    pub html_bytes: usize,
    pub visible_text_chars: usize,
    pub p_count: u32,
    pub article_count: u32,
    pub h1h2_count: u32,
    pub script_bytes: usize,
    pub has_og: bool,
    pub has_jsonld_article: bool,
    pub spa_markers: u8, // bit flags: NEXT_DATA, hydration, root div, huge bundle
    pub domain_prior: f32, // 0.0..1.0
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    Raw,
    ProbesFirst,
    Headless,
}

pub fn score(features: &GateFeatures) -> f32 {
    let text_ratio = if features.html_bytes == 0 {
        0.0
    } else {
        features.visible_text_chars as f32 / features.html_bytes as f32
    };
    let script_density = if features.html_bytes == 0 {
        0.0
    } else {
        features.script_bytes as f32 / features.html_bytes as f32
    };
    let mut s = 0.0;
    s += (text_ratio * 1.2).clamp(0.0, 0.6);
    s += ((features.p_count as f32 + 1.0).ln() * 0.06).clamp(0.0, 0.3);
    if features.article_count > 0 {
        s += 0.15;
    }
    if features.has_og {
        s += 0.08;
    }
    if features.has_jsonld_article {
        s += 0.12;
    }
    s -= (script_density * 0.8).clamp(0.0, 0.4);
    if features.spa_markers >= 2 {
        s -= 0.25;
    }
    (s + (features.domain_prior - 0.5) * 0.1).clamp(0.0, 1.0)
}

pub fn decide(features: &GateFeatures, hi: f32, lo: f32) -> Decision {
    let s = score(features);
    if s >= hi {
        Decision::Raw
    } else if s <= lo || features.spa_markers >= 3 {
        Decision::Headless
    } else {
        Decision::ProbesFirst
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_simple_article() {
        let features = GateFeatures {
            html_bytes: 10000,
            visible_text_chars: 5000,
            p_count: 10,
            article_count: 1,
            h1h2_count: 3,
            script_bytes: 500,
            has_og: true,
            has_jsonld_article: true,
            spa_markers: 0,
            domain_prior: 0.7,
        };
        let s = score(&features);
        assert!(s > 0.5);
    }

    #[test]
    fn test_decide_spa() {
        let features = GateFeatures {
            html_bytes: 10000,
            visible_text_chars: 500,
            p_count: 2,
            article_count: 0,
            h1h2_count: 1,
            script_bytes: 8000,
            has_og: false,
            has_jsonld_article: false,
            spa_markers: 3,
            domain_prior: 0.5,
        };
        assert_eq!(decide(&features, 0.7, 0.3), Decision::Headless);
    }
}
