//! JavaScript injection for stealth mode
//!
//! This module generates and manages JavaScript code that gets injected
//! into pages to override various browser APIs and prevent detection.

use crate::stealth::config::LocaleStrategy;
use crate::stealth::fingerprint::{CanvasConfig, HardwareConfig, WebGlConfig};

/// JavaScript injector for stealth countermeasures
pub struct JavaScriptInjector {
    /// Locale for the session
    locale: String,
    /// Timezone for the session
    timezone: String,
    /// CPU cores to report
    cpu_cores: u32,
    /// Device memory to report (in GB)
    device_memory: u32,
    /// WebGL vendor string
    webgl_vendor: String,
    /// WebGL renderer string
    webgl_renderer: String,
    /// Canvas configuration
    canvas_config: CanvasConfig,
}

impl JavaScriptInjector {
    /// Create a new JavaScript injector with random values
    pub fn new(
        hardware_config: &HardwareConfig,
        webgl_config: &WebGlConfig,
        canvas_config: &CanvasConfig,
        locale_strategy: &LocaleStrategy,
    ) -> Self {
        let (locale, timezone) = Self::generate_locale(locale_strategy);
        let (cpu_cores, device_memory) = hardware_config.get_random_hardware_specs();
        let (webgl_vendor, webgl_renderer) = webgl_config.get_random_webgl_specs();

        Self {
            locale,
            timezone,
            cpu_cores,
            device_memory,
            webgl_vendor,
            webgl_renderer,
            canvas_config: canvas_config.clone(),
        }
    }

    /// Generate the complete stealth JavaScript injection
    pub fn generate_stealth_js(&self) -> String {
        format!(
            r#"
// Stealth JavaScript injection - injected early in page lifecycle
(function() {{
    'use strict';

    // Store original functions before overriding
    const originalDescriptor = Object.getOwnPropertyDescriptor;
    const originalDefineProperty = Object.defineProperty;

    {}

    {}

    {}

    {}

    {}

    {}

    {}

    {}

    {}
}})();
"#,
            self.generate_webdriver_override(),
            self.generate_plugins_override(),
            self.generate_language_override(),
            self.generate_hardware_override(),
            self.generate_webgl_override(),
            self.generate_canvas_protection(),
            self.generate_automation_cleanup(),
            self.generate_timezone_override(),
            self.generate_additional_protections()
        )
    }

    /// Generate webdriver detection override
    fn generate_webdriver_override(&self) -> String {
        r#"
    // Override webdriver detection
    originalDefineProperty(navigator, 'webdriver', {
        get: () => false,
        enumerable: true,
        configurable: true
    });

    // Remove webdriver from toString
    const navigatorToString = navigator.toString;
    navigator.toString = function() {
        return navigatorToString.call(this).replace(/webdriver/gi, '');
    };"#
            .to_string()
    }

    /// Generate plugins override
    fn generate_plugins_override(&self) -> String {
        r#"
    // Override navigator.plugins to appear natural
    const mockPlugins = [
        {name: 'Chrome PDF Plugin', filename: 'internal-pdf-viewer', description: 'Portable Document Format'},
        {name: 'Chrome PDF Viewer', filename: 'mhjfbmdgcfjbbpaeojofohoefgiehjai', description: 'Portable Document Format'},
        {name: 'Native Client', filename: 'internal-nacl-plugin', description: 'Native Client'}
    ];

    originalDefineProperty(navigator, 'plugins', {
        get: () => mockPlugins,
        enumerable: true,
        configurable: true
    });

    originalDefineProperty(navigator, 'mimeTypes', {
        get: () => ({
            'application/pdf': {type: 'application/pdf', suffixes: 'pdf', enabledPlugin: mockPlugins[0]},
            length: 1
        }),
        enumerable: true,
        configurable: true
    });"#
            .to_string()
    }

    /// Generate language and locale override
    fn generate_language_override(&self) -> String {
        format!(
            r#"
    // Override languages and locale
    originalDefineProperty(navigator, 'languages', {{
        get: () => ['{}', 'en'],
        enumerable: true,
        configurable: true
    }});

    originalDefineProperty(navigator, 'language', {{
        get: () => '{}',
        enumerable: true,
        configurable: true
    }});"#,
            self.locale, self.locale
        )
    }

    /// Generate hardware specifications override
    fn generate_hardware_override(&self) -> String {
        format!(
            r#"
    // Hardware spoofing
    originalDefineProperty(navigator, 'hardwareConcurrency', {{
        get: () => {},
        enumerable: true,
        configurable: true
    }});

    originalDefineProperty(navigator, 'deviceMemory', {{
        get: () => {},
        enumerable: true,
        configurable: true
    }});

    // Platform override for consistency
    originalDefineProperty(navigator, 'platform', {{
        get: () => 'Win32',
        enumerable: true,
        configurable: true
    }});"#,
            self.cpu_cores, self.device_memory
        )
    }

    /// Generate WebGL vendor spoofing
    fn generate_webgl_override(&self) -> String {
        format!(
            r#"
    // WebGL vendor spoofing
    const getParameter = WebGLRenderingContext.prototype.getParameter;
    WebGLRenderingContext.prototype.getParameter = function(parameter) {{
        if (parameter === 37445) return '{}'; // UNMASKED_VENDOR_WEBGL
        if (parameter === 37446) return '{}'; // UNMASKED_RENDERER_WEBGL
        return getParameter.apply(this, arguments);
    }};

    // WebGL2 support
    if (typeof WebGL2RenderingContext !== 'undefined') {{
        const getParameter2 = WebGL2RenderingContext.prototype.getParameter;
        WebGL2RenderingContext.prototype.getParameter = function(parameter) {{
            if (parameter === 37445) return '{}';
            if (parameter === 37446) return '{}';
            return getParameter2.apply(this, arguments);
        }};
    }}"#,
            self.webgl_vendor, self.webgl_renderer, self.webgl_vendor, self.webgl_renderer
        )
    }

    /// Generate canvas fingerprint protection
    fn generate_canvas_protection(&self) -> String {
        if !self.canvas_config.add_noise {
            return String::new();
        }

        format!(
            r#"
    // Canvas fingerprint noise injection
    const originalToDataURL = HTMLCanvasElement.prototype.toDataURL;
    const originalGetImageData = CanvasRenderingContext2D.prototype.getImageData;

    HTMLCanvasElement.prototype.toDataURL = function(...args) {{
        const context = this.getContext('2d');
        if (context) {{
            const originalImageData = originalGetImageData.call(context, 0, 0, this.width, this.height);
            const imageData = new ImageData(
                new Uint8ClampedArray(originalImageData.data),
                originalImageData.width,
                originalImageData.height
            );

            const noise = {};
            for (let i = 0; i < imageData.data.length; i += 4) {{
                imageData.data[i] += Math.floor((Math.random() - 0.5) * noise);
                imageData.data[i+1] += Math.floor((Math.random() - 0.5) * noise);
                imageData.data[i+2] += Math.floor((Math.random() - 0.5) * noise);
                // Keep alpha channel unchanged
            }}
            context.putImageData(imageData, 0, 0);
        }}
        return originalToDataURL.apply(this, args);
    }};

    CanvasRenderingContext2D.prototype.getImageData = function(...args) {{
        const imageData = originalGetImageData.apply(this, args);
        const noise = {};
        for (let i = 0; i < imageData.data.length; i += 4) {{
            imageData.data[i] += Math.floor((Math.random() - 0.5) * noise);
            imageData.data[i+1] += Math.floor((Math.random() - 0.5) * noise);
            imageData.data[i+2] += Math.floor((Math.random() - 0.5) * noise);
        }}
        return imageData;
    }};"#,
            self.canvas_config.noise_intensity * 255.0,
            self.canvas_config.noise_intensity * 255.0
        )
    }

    /// Generate automation detection cleanup
    fn generate_automation_cleanup(&self) -> String {
        r#"
    // Hide automation-specific properties
    const automationProps = [
        '__webdriver_evaluate', '__webdriver_script_func', '__webdriver_script_fn',
        '__fxdriver_evaluate', '__fxdriver_unwrapped', '__driver_evaluate',
        '__webdriver_unwrapped', '__driver_unwrapped', '__selenium_unwrapped',
        '__webdriver_script_function', '__webdriver_script_func'
    ];

    automationProps.forEach(prop => {
        delete navigator[prop];
        delete window[prop];
    });

    // Remove automation flags from window
    const windowProps = ['__nightmare', '_phantom', 'Buffer', 'emit', 'spawn', 'webdriver'];
    windowProps.forEach(prop => {
        delete window[prop];
    });

    // Override toString methods to remove automation traces
    const descriptors = ['webdriver', 'driver', 'selenium'];
    descriptors.forEach(desc => {
        if (window[desc]) {
            delete window[desc];
        }
    });"#
            .to_string()
    }

    /// Generate timezone override
    fn generate_timezone_override(&self) -> String {
        let timezone_offset = self.get_timezone_offset(&self.timezone);
        format!(
            r#"
    // Timezone override
    const originalGetTimezoneOffset = Date.prototype.getTimezoneOffset;
    Date.prototype.getTimezoneOffset = function() {{
        return {};
    }};

    // Intl.DateTimeFormat override
    if (typeof Intl !== 'undefined' && Intl.DateTimeFormat) {{
        const originalResolvedOptions = Intl.DateTimeFormat.prototype.resolvedOptions;
        Intl.DateTimeFormat.prototype.resolvedOptions = function() {{
            const options = originalResolvedOptions.call(this);
            options.timeZone = '{}';
            return options;
        }};
    }}"#,
            timezone_offset, self.timezone
        )
    }

    /// Generate additional protection measures
    fn generate_additional_protections(&self) -> String {
        r#"
    // Screen and media device spoofing
    originalDefineProperty(screen, 'colorDepth', {
        get: () => 24,
        enumerable: true,
        configurable: true
    });

    originalDefineProperty(screen, 'pixelDepth', {
        get: () => 24,
        enumerable: true,
        configurable: true
    });

    // Battery API spoofing
    if ('getBattery' in navigator) {
        navigator.getBattery = async () => ({
            charging: true,
            chargingTime: 0,
            dischargingTime: Infinity,
            level: Math.random() * 0.3 + 0.6 // 60-90%
        });
    }

    // Audio context fingerprinting protection
    if (typeof AudioContext !== 'undefined') {
        const OriginalAudioContext = AudioContext;
        window.AudioContext = function(...args) {
            const context = new OriginalAudioContext(...args);
            const originalCreateAnalyser = context.createAnalyser;
            context.createAnalyser = function() {
                const analyser = originalCreateAnalyser.call(this);
                const originalGetFloatFrequencyData = analyser.getFloatFrequencyData;
                analyser.getFloatFrequencyData = function(array) {
                    originalGetFloatFrequencyData.call(this, array);
                    // Add slight noise to audio fingerprinting
                    for (let i = 0; i < array.length; i++) {
                        array[i] += (Math.random() - 0.5) * 0.001;
                    }
                };
                return analyser;
            };
            return context;
        };
    }"#
            .to_string()
    }

    /// Generate locale based on strategy
    fn generate_locale(strategy: &LocaleStrategy) -> (String, String) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let locales_and_timezones = vec![
            ("en-US", "America/New_York"),
            ("en-GB", "Europe/London"),
            ("de-DE", "Europe/Berlin"),
            ("fr-FR", "Europe/Paris"),
            ("es-ES", "Europe/Madrid"),
            ("it-IT", "Europe/Rome"),
        ];

        match strategy {
            LocaleStrategy::Random => {
                let index = rng.gen_range(0..locales_and_timezones.len());
                let (locale, timezone) = locales_and_timezones[index];
                (locale.to_string(), timezone.to_string())
            }
            LocaleStrategy::Fixed(locale) => {
                let timezone = match locale.as_str() {
                    "en-US" => "America/New_York",
                    "en-GB" => "Europe/London",
                    "de-DE" => "Europe/Berlin",
                    "fr-FR" => "Europe/Paris",
                    "es-ES" => "Europe/Madrid",
                    "it-IT" => "Europe/Rome",
                    _ => "America/New_York",
                };
                (locale.clone(), timezone.to_string())
            }
            _ => ("en-US".to_string(), "America/New_York".to_string()),
        }
    }

    /// Get timezone offset for given timezone
    fn get_timezone_offset(&self, timezone: &str) -> i32 {
        match timezone {
            "America/New_York" => 300,    // EST
            "America/Chicago" => 360,     // CST
            "America/Denver" => 420,      // MST
            "America/Los_Angeles" => 480, // PST
            "Europe/London" => 0,         // GMT
            "Europe/Berlin" => -60,       // CET
            "Europe/Paris" => -60,        // CET
            "Europe/Madrid" => -60,       // CET
            "Europe/Rome" => -60,         // CET
            "Asia/Tokyo" => -540,         // JST
            "Asia/Shanghai" => -480,      // CST
            _ => 300,                     // Default to EST
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stealth::fingerprint::{CanvasConfig, HardwareConfig, WebGlConfig};

    #[test]
    fn test_javascript_injector_creation() {
        let hardware_config = HardwareConfig::default();
        let webgl_config = WebGlConfig::default();
        let canvas_config = CanvasConfig::default();
        let locale_strategy = LocaleStrategy::Random;

        let injector = JavaScriptInjector::new(
            &hardware_config,
            &webgl_config,
            &canvas_config,
            &locale_strategy,
        );

        assert!(!injector.locale.is_empty());
        assert!(!injector.timezone.is_empty());
        assert!(injector.cpu_cores > 0);
        assert!(injector.device_memory > 0);
    }

    #[test]
    fn test_stealth_js_generation() {
        let hardware_config = HardwareConfig::default();
        let webgl_config = WebGlConfig::default();
        let canvas_config = CanvasConfig::default();
        let locale_strategy = LocaleStrategy::Fixed("en-US".to_string());

        let injector = JavaScriptInjector::new(
            &hardware_config,
            &webgl_config,
            &canvas_config,
            &locale_strategy,
        );

        let js_code = injector.generate_stealth_js();

        // Verify key components are included
        assert!(js_code.contains("'webdriver'"));
        assert!(js_code.contains("hardwareConcurrency"));
        assert!(js_code.contains("deviceMemory"));
        assert!(js_code.contains("WebGLRenderingContext"));
        assert!(js_code.contains("en-US"));
        assert!(!js_code.is_empty());
    }

    #[test]
    fn test_timezone_offset_calculation() {
        let hardware_config = HardwareConfig::default();
        let webgl_config = WebGlConfig::default();
        let canvas_config = CanvasConfig::default();
        let locale_strategy = LocaleStrategy::Fixed("en-US".to_string());

        let injector = JavaScriptInjector::new(
            &hardware_config,
            &webgl_config,
            &canvas_config,
            &locale_strategy,
        );

        assert_eq!(injector.get_timezone_offset("America/New_York"), 300);
        assert_eq!(injector.get_timezone_offset("Europe/London"), 0);
        assert_eq!(injector.get_timezone_offset("Europe/Berlin"), -60);
        assert_eq!(injector.get_timezone_offset("Unknown"), 300); // Default
    }
}