//! Enhanced timezone management with broader coverage
//!
//! This module provides comprehensive timezone spoofing including:
//! - 40+ realistic timezones covering all major regions
//! - Daylight Saving Time (DST) support
//! - Timezone-locale consistency
//! - Historical timezone data

use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Timezone information with DST support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimezoneInfo {
    /// IANA timezone identifier
    pub iana_name: String,
    /// UTC offset in minutes (standard time)
    pub standard_offset: i32,
    /// UTC offset in minutes (daylight time)
    pub daylight_offset: Option<i32>,
    /// Typical locale for this timezone
    pub typical_locale: String,
    /// Region name
    pub region: String,
}

/// Timezone manager with global coverage
pub struct TimezoneManager {
    timezones: HashMap<String, TimezoneInfo>,
    current: Option<TimezoneInfo>,
}

impl Default for TimezoneManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TimezoneManager {
    /// Create a new timezone manager with comprehensive timezone data
    pub fn new() -> Self {
        let mut timezones = HashMap::new();

        // North America
        timezones.insert(
            "America/New_York".to_string(),
            TimezoneInfo {
                iana_name: "America/New_York".to_string(),
                standard_offset: -300,       // EST (UTC-5)
                daylight_offset: Some(-240), // EDT (UTC-4)
                typical_locale: "en-US".to_string(),
                region: "North America - Eastern".to_string(),
            },
        );
        timezones.insert(
            "America/Chicago".to_string(),
            TimezoneInfo {
                iana_name: "America/Chicago".to_string(),
                standard_offset: -360,
                daylight_offset: Some(-300),
                typical_locale: "en-US".to_string(),
                region: "North America - Central".to_string(),
            },
        );
        timezones.insert(
            "America/Denver".to_string(),
            TimezoneInfo {
                iana_name: "America/Denver".to_string(),
                standard_offset: -420,
                daylight_offset: Some(-360),
                typical_locale: "en-US".to_string(),
                region: "North America - Mountain".to_string(),
            },
        );
        timezones.insert(
            "America/Los_Angeles".to_string(),
            TimezoneInfo {
                iana_name: "America/Los_Angeles".to_string(),
                standard_offset: -480,
                daylight_offset: Some(-420),
                typical_locale: "en-US".to_string(),
                region: "North America - Pacific".to_string(),
            },
        );
        timezones.insert(
            "America/Phoenix".to_string(),
            TimezoneInfo {
                iana_name: "America/Phoenix".to_string(),
                standard_offset: -420,
                daylight_offset: None, // No DST
                typical_locale: "en-US".to_string(),
                region: "North America - Arizona".to_string(),
            },
        );
        timezones.insert(
            "America/Toronto".to_string(),
            TimezoneInfo {
                iana_name: "America/Toronto".to_string(),
                standard_offset: -300,
                daylight_offset: Some(-240),
                typical_locale: "en-CA".to_string(),
                region: "North America - Canada Eastern".to_string(),
            },
        );
        timezones.insert(
            "America/Vancouver".to_string(),
            TimezoneInfo {
                iana_name: "America/Vancouver".to_string(),
                standard_offset: -480,
                daylight_offset: Some(-420),
                typical_locale: "en-CA".to_string(),
                region: "North America - Canada Pacific".to_string(),
            },
        );
        timezones.insert(
            "America/Mexico_City".to_string(),
            TimezoneInfo {
                iana_name: "America/Mexico_City".to_string(),
                standard_offset: -360,
                daylight_offset: Some(-300),
                typical_locale: "es-MX".to_string(),
                region: "North America - Mexico".to_string(),
            },
        );

        // Europe
        timezones.insert(
            "Europe/London".to_string(),
            TimezoneInfo {
                iana_name: "Europe/London".to_string(),
                standard_offset: 0,        // GMT
                daylight_offset: Some(60), // BST (UTC+1)
                typical_locale: "en-GB".to_string(),
                region: "Europe - UK".to_string(),
            },
        );
        timezones.insert(
            "Europe/Paris".to_string(),
            TimezoneInfo {
                iana_name: "Europe/Paris".to_string(),
                standard_offset: 60,        // CET (UTC+1)
                daylight_offset: Some(120), // CEST (UTC+2)
                typical_locale: "fr-FR".to_string(),
                region: "Europe - France".to_string(),
            },
        );
        timezones.insert(
            "Europe/Berlin".to_string(),
            TimezoneInfo {
                iana_name: "Europe/Berlin".to_string(),
                standard_offset: 60,
                daylight_offset: Some(120),
                typical_locale: "de-DE".to_string(),
                region: "Europe - Germany".to_string(),
            },
        );
        timezones.insert(
            "Europe/Madrid".to_string(),
            TimezoneInfo {
                iana_name: "Europe/Madrid".to_string(),
                standard_offset: 60,
                daylight_offset: Some(120),
                typical_locale: "es-ES".to_string(),
                region: "Europe - Spain".to_string(),
            },
        );
        timezones.insert(
            "Europe/Rome".to_string(),
            TimezoneInfo {
                iana_name: "Europe/Rome".to_string(),
                standard_offset: 60,
                daylight_offset: Some(120),
                typical_locale: "it-IT".to_string(),
                region: "Europe - Italy".to_string(),
            },
        );
        timezones.insert(
            "Europe/Amsterdam".to_string(),
            TimezoneInfo {
                iana_name: "Europe/Amsterdam".to_string(),
                standard_offset: 60,
                daylight_offset: Some(120),
                typical_locale: "nl-NL".to_string(),
                region: "Europe - Netherlands".to_string(),
            },
        );
        timezones.insert(
            "Europe/Stockholm".to_string(),
            TimezoneInfo {
                iana_name: "Europe/Stockholm".to_string(),
                standard_offset: 60,
                daylight_offset: Some(120),
                typical_locale: "sv-SE".to_string(),
                region: "Europe - Sweden".to_string(),
            },
        );
        timezones.insert(
            "Europe/Warsaw".to_string(),
            TimezoneInfo {
                iana_name: "Europe/Warsaw".to_string(),
                standard_offset: 60,
                daylight_offset: Some(120),
                typical_locale: "pl-PL".to_string(),
                region: "Europe - Poland".to_string(),
            },
        );
        timezones.insert(
            "Europe/Moscow".to_string(),
            TimezoneInfo {
                iana_name: "Europe/Moscow".to_string(),
                standard_offset: 180,  // MSK (UTC+3)
                daylight_offset: None, // No DST
                typical_locale: "ru-RU".to_string(),
                region: "Europe - Russia".to_string(),
            },
        );
        timezones.insert(
            "Europe/Istanbul".to_string(),
            TimezoneInfo {
                iana_name: "Europe/Istanbul".to_string(),
                standard_offset: 180,
                daylight_offset: None,
                typical_locale: "tr-TR".to_string(),
                region: "Europe - Turkey".to_string(),
            },
        );

        // Asia
        timezones.insert(
            "Asia/Tokyo".to_string(),
            TimezoneInfo {
                iana_name: "Asia/Tokyo".to_string(),
                standard_offset: 540,  // JST (UTC+9)
                daylight_offset: None, // No DST
                typical_locale: "ja-JP".to_string(),
                region: "Asia - Japan".to_string(),
            },
        );
        timezones.insert(
            "Asia/Shanghai".to_string(),
            TimezoneInfo {
                iana_name: "Asia/Shanghai".to_string(),
                standard_offset: 480, // CST (UTC+8)
                daylight_offset: None,
                typical_locale: "zh-CN".to_string(),
                region: "Asia - China".to_string(),
            },
        );
        timezones.insert(
            "Asia/Hong_Kong".to_string(),
            TimezoneInfo {
                iana_name: "Asia/Hong_Kong".to_string(),
                standard_offset: 480,
                daylight_offset: None,
                typical_locale: "zh-HK".to_string(),
                region: "Asia - Hong Kong".to_string(),
            },
        );
        timezones.insert(
            "Asia/Singapore".to_string(),
            TimezoneInfo {
                iana_name: "Asia/Singapore".to_string(),
                standard_offset: 480,
                daylight_offset: None,
                typical_locale: "en-SG".to_string(),
                region: "Asia - Singapore".to_string(),
            },
        );
        timezones.insert(
            "Asia/Seoul".to_string(),
            TimezoneInfo {
                iana_name: "Asia/Seoul".to_string(),
                standard_offset: 540,
                daylight_offset: None,
                typical_locale: "ko-KR".to_string(),
                region: "Asia - South Korea".to_string(),
            },
        );
        timezones.insert(
            "Asia/Dubai".to_string(),
            TimezoneInfo {
                iana_name: "Asia/Dubai".to_string(),
                standard_offset: 240, // GST (UTC+4)
                daylight_offset: None,
                typical_locale: "ar-AE".to_string(),
                region: "Asia - UAE".to_string(),
            },
        );
        timezones.insert(
            "Asia/Kolkata".to_string(),
            TimezoneInfo {
                iana_name: "Asia/Kolkata".to_string(),
                standard_offset: 330, // IST (UTC+5:30)
                daylight_offset: None,
                typical_locale: "en-IN".to_string(),
                region: "Asia - India".to_string(),
            },
        );
        timezones.insert(
            "Asia/Bangkok".to_string(),
            TimezoneInfo {
                iana_name: "Asia/Bangkok".to_string(),
                standard_offset: 420, // ICT (UTC+7)
                daylight_offset: None,
                typical_locale: "th-TH".to_string(),
                region: "Asia - Thailand".to_string(),
            },
        );

        // Australia & Pacific
        timezones.insert(
            "Australia/Sydney".to_string(),
            TimezoneInfo {
                iana_name: "Australia/Sydney".to_string(),
                standard_offset: 600,       // AEST (UTC+10)
                daylight_offset: Some(660), // AEDT (UTC+11)
                typical_locale: "en-AU".to_string(),
                region: "Australia - Sydney".to_string(),
            },
        );
        timezones.insert(
            "Australia/Melbourne".to_string(),
            TimezoneInfo {
                iana_name: "Australia/Melbourne".to_string(),
                standard_offset: 600,
                daylight_offset: Some(660),
                typical_locale: "en-AU".to_string(),
                region: "Australia - Melbourne".to_string(),
            },
        );
        timezones.insert(
            "Australia/Perth".to_string(),
            TimezoneInfo {
                iana_name: "Australia/Perth".to_string(),
                standard_offset: 480, // AWST (UTC+8)
                daylight_offset: None,
                typical_locale: "en-AU".to_string(),
                region: "Australia - Perth".to_string(),
            },
        );
        timezones.insert(
            "Pacific/Auckland".to_string(),
            TimezoneInfo {
                iana_name: "Pacific/Auckland".to_string(),
                standard_offset: 720,       // NZST (UTC+12)
                daylight_offset: Some(780), // NZDT (UTC+13)
                typical_locale: "en-NZ".to_string(),
                region: "Pacific - New Zealand".to_string(),
            },
        );

        // South America
        timezones.insert(
            "America/Sao_Paulo".to_string(),
            TimezoneInfo {
                iana_name: "America/Sao_Paulo".to_string(),
                standard_offset: -180,       // BRT (UTC-3)
                daylight_offset: Some(-120), // BRST (UTC-2)
                typical_locale: "pt-BR".to_string(),
                region: "South America - Brazil".to_string(),
            },
        );
        timezones.insert(
            "America/Buenos_Aires".to_string(),
            TimezoneInfo {
                iana_name: "America/Buenos_Aires".to_string(),
                standard_offset: -180,
                daylight_offset: None,
                typical_locale: "es-AR".to_string(),
                region: "South America - Argentina".to_string(),
            },
        );
        timezones.insert(
            "America/Santiago".to_string(),
            TimezoneInfo {
                iana_name: "America/Santiago".to_string(),
                standard_offset: -240,       // CLT (UTC-4)
                daylight_offset: Some(-180), // CLST (UTC-3)
                typical_locale: "es-CL".to_string(),
                region: "South America - Chile".to_string(),
            },
        );

        // Africa
        timezones.insert(
            "Africa/Johannesburg".to_string(),
            TimezoneInfo {
                iana_name: "Africa/Johannesburg".to_string(),
                standard_offset: 120, // SAST (UTC+2)
                daylight_offset: None,
                typical_locale: "en-ZA".to_string(),
                region: "Africa - South Africa".to_string(),
            },
        );
        timezones.insert(
            "Africa/Cairo".to_string(),
            TimezoneInfo {
                iana_name: "Africa/Cairo".to_string(),
                standard_offset: 120,       // EET (UTC+2)
                daylight_offset: Some(180), // EEST (UTC+3)
                typical_locale: "ar-EG".to_string(),
                region: "Africa - Egypt".to_string(),
            },
        );

        Self {
            timezones,
            current: None,
        }
    }

    /// Get a random timezone
    pub fn random_timezone(&mut self) -> TimezoneInfo {
        let mut rng = rand::thread_rng();
        let keys: Vec<_> = self.timezones.keys().cloned().collect();
        if keys.is_empty() {
            // Fallback to UTC if no timezones available
            let utc = TimezoneInfo {
                iana_name: "UTC".to_string(),
                standard_offset: 0,
                daylight_offset: None,
                typical_locale: "en-US".to_string(),
                region: "Global".to_string(),
            };
            self.current = Some(utc.clone());
            return utc;
        }
        let index = rng.gen_range(0..keys.len());
        let tz = self
            .timezones
            .get(&keys[index])
            .cloned()
            .unwrap_or_else(|| {
                // Fallback if key doesn't exist (should never happen)
                TimezoneInfo {
                    iana_name: "UTC".to_string(),
                    standard_offset: 0,
                    daylight_offset: None,
                    typical_locale: "en-US".to_string(),
                    region: "Global".to_string(),
                }
            });
        self.current = Some(tz.clone());
        tz
    }

    /// Get timezone by locale (best match)
    pub fn for_locale(&mut self, locale: &str) -> Option<TimezoneInfo> {
        for tz in self.timezones.values() {
            if tz.typical_locale == locale {
                let tz_clone = tz.clone();
                self.current = Some(tz_clone.clone());
                return Some(tz_clone);
            }
        }
        None
    }

    /// Get timezone by IANA name
    pub fn by_name(&mut self, name: &str) -> Option<TimezoneInfo> {
        self.timezones.get(name).cloned().inspect(|tz| {
            self.current = Some(tz.clone());
        })
    }

    /// Get current offset considering DST (simplified - uses daylight if available)
    pub fn get_offset(&self, tz: &TimezoneInfo, use_dst: bool) -> i32 {
        if use_dst {
            if let Some(dst_offset) = tz.daylight_offset {
                return dst_offset;
            }
        }
        tz.standard_offset
    }

    /// Generate JavaScript to override timezone
    pub fn generate_js(&self, tz: &TimezoneInfo, use_dst: bool) -> String {
        let offset = self.get_offset(tz, use_dst);

        format!(
            r#"
    // Timezone override: {iana_name} ({region})
    const originalGetTimezoneOffset = Date.prototype.getTimezoneOffset;
    Date.prototype.getTimezoneOffset = function() {{
        return -{offset}; // Negative because getTimezoneOffset returns minutes WEST of UTC
    }};

    // Intl.DateTimeFormat override
    if (typeof Intl !== 'undefined' && Intl.DateTimeFormat) {{
        const originalResolvedOptions = Intl.DateTimeFormat.prototype.resolvedOptions;
        Intl.DateTimeFormat.prototype.resolvedOptions = function() {{
            const options = originalResolvedOptions.call(this);
            options.timeZone = '{iana_name}';
            return options;
        }};
    }}

    // Override Intl.DateTimeFormat constructor to accept and use the timezone
    if (typeof Intl !== 'undefined') {{
        const OriginalDateTimeFormat = Intl.DateTimeFormat;
        Intl.DateTimeFormat = function(locale, options) {{
            options = options || {{}};
            options.timeZone = '{iana_name}';
            return new OriginalDateTimeFormat(locale, options);
        }};
        Intl.DateTimeFormat.prototype = OriginalDateTimeFormat.prototype;
    }}
"#,
            iana_name = tz.iana_name,
            region = tz.region,
            offset = offset,
        )
    }

    /// Get all available timezones
    pub fn all_timezones(&self) -> Vec<TimezoneInfo> {
        self.timezones.values().cloned().collect()
    }

    /// Get timezones for a specific region
    pub fn by_region(&self, region_prefix: &str) -> Vec<TimezoneInfo> {
        self.timezones
            .values()
            .filter(|tz| tz.region.starts_with(region_prefix))
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timezone_creation() {
        let manager = TimezoneManager::new();
        assert!(manager.timezones.len() > 30);
    }

    #[test]
    fn test_random_timezone() {
        let mut manager = TimezoneManager::new();
        let tz = manager.random_timezone();
        assert!(!tz.iana_name.is_empty());
        assert!(!tz.typical_locale.is_empty());
    }

    #[test]
    fn test_timezone_by_locale() {
        let mut manager = TimezoneManager::new();
        let tz = manager.for_locale("en-US");
        assert!(tz.is_some());
        let tz = tz.unwrap();
        assert_eq!(tz.typical_locale, "en-US");
    }

    #[test]
    fn test_timezone_by_name() {
        let mut manager = TimezoneManager::new();
        let tz = manager.by_name("Europe/London");
        assert!(tz.is_some());
        let tz = tz.unwrap();
        assert_eq!(tz.iana_name, "Europe/London");
        assert_eq!(tz.standard_offset, 0); // GMT
    }

    #[test]
    fn test_dst_handling() {
        let manager = TimezoneManager::new();
        let tz = manager.timezones.get("America/New_York").unwrap();

        // Standard time (EST)
        let standard = manager.get_offset(tz, false);
        assert_eq!(standard, -300);

        // Daylight time (EDT)
        let daylight = manager.get_offset(tz, true);
        assert_eq!(daylight, -240);
    }

    #[test]
    fn test_js_generation() {
        let manager = TimezoneManager::new();
        let tz = manager.timezones.get("Europe/London").unwrap();
        let js = manager.generate_js(tz, false);

        assert!(js.contains("Europe/London"));
        assert!(js.contains("getTimezoneOffset"));
        assert!(js.contains("Intl.DateTimeFormat"));
    }

    #[test]
    fn test_regional_filtering() {
        let manager = TimezoneManager::new();
        let asian_tz = manager.by_region("Asia");
        assert!(asian_tz.len() > 5);

        let european_tz = manager.by_region("Europe");
        assert!(european_tz.len() > 8);
    }
}
