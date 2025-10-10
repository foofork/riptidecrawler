//! Enhanced screen resolution management with consistency
//!
//! This module provides advanced screen resolution randomization that maintains
//! consistency across related properties (window size, screen size, available size)
//! to avoid fingerprinting detection.

use rand::Rng;
use serde::{Deserialize, Serialize};

/// Screen resolution with consistent related properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenResolution {
    /// Full screen width
    pub screen_width: u32,
    /// Full screen height
    pub screen_height: u32,
    /// Available width (excluding taskbar)
    pub avail_width: u32,
    /// Available height (excluding taskbar)
    pub avail_height: u32,
    /// Window outer width
    pub outer_width: u32,
    /// Window outer height
    pub outer_height: u32,
    /// Window inner width
    pub inner_width: u32,
    /// Window inner height
    pub inner_height: u32,
    /// Color depth (always 24 for modern displays)
    pub color_depth: u8,
    /// Pixel depth (always 24 for modern displays)
    pub pixel_depth: u8,
    /// Device pixel ratio (1.0, 1.25, 1.5, 2.0)
    pub device_pixel_ratio: f32,
}

/// Screen resolution manager for consistent randomization
pub struct ScreenResolutionManager {
    /// Common realistic resolutions
    resolutions: Vec<(u32, u32)>,
    /// Current resolution for session
    current: Option<ScreenResolution>,
}

impl Default for ScreenResolutionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenResolutionManager {
    /// Create a new screen resolution manager with realistic presets
    pub fn new() -> Self {
        Self {
            resolutions: vec![
                // 16:9 Resolutions (most common)
                (1920, 1080), // Full HD - most common
                (1366, 768),  // HD - common laptop
                (1536, 864),  // HD+
                (1600, 900),  // HD+
                (2560, 1440), // QHD/2K
                (3840, 2160), // 4K
                // 16:10 Resolutions (MacBook, professional displays)
                (1440, 900),  // MacBook Air
                (1680, 1050), // 20" display
                (1920, 1200), // 24" display
                (2560, 1600), // MacBook Pro 13"
                (3072, 1920), // MacBook Pro 16"
                // 21:9 Ultrawide
                (2560, 1080), // Ultrawide HD
                (3440, 1440), // Ultrawide QHD
            ],
            current: None,
        }
    }

    /// Generate a realistic screen resolution with consistent properties
    pub fn generate(&mut self) -> ScreenResolution {
        let mut rng = rand::thread_rng();

        // Pick a random resolution
        let index = rng.gen_range(0..self.resolutions.len());
        let (screen_width, screen_height) = self.resolutions[index];

        // Realistic device pixel ratios
        let pixel_ratios = [1.0, 1.25, 1.5, 2.0];
        let device_pixel_ratio = pixel_ratios[rng.gen_range(0..pixel_ratios.len())];

        // Available size = screen size minus taskbar (typically 40-50 pixels on Windows, 25 on macOS)
        let taskbar_height = if rng.gen_bool(0.7) {
            // Windows taskbar
            40
        } else {
            // macOS menu bar + dock
            25
        };

        let avail_width = screen_width;
        let avail_height = screen_height - taskbar_height;

        // Window size: typically maximized or slightly smaller
        let (outer_width, outer_height) = if rng.gen_bool(0.8) {
            // Maximized window
            (avail_width, avail_height)
        } else {
            // Non-maximized window (80-95% of available space)
            let width_factor = 0.8 + (rng.gen::<f32>() * 0.15);
            let height_factor = 0.8 + (rng.gen::<f32>() * 0.15);
            (
                (avail_width as f32 * width_factor) as u32,
                (avail_height as f32 * height_factor) as u32,
            )
        };

        // Inner size = outer size minus browser chrome (address bar, tabs, etc)
        // Chrome height: typically 120-140 pixels
        let chrome_height: u32 = rng.gen_range(120..140);
        let chrome_width: u32 = 0; // No horizontal chrome

        let inner_width = outer_width - chrome_width;
        let inner_height = outer_height - chrome_height;

        let resolution = ScreenResolution {
            screen_width,
            screen_height,
            avail_width,
            avail_height,
            outer_width,
            outer_height,
            inner_width,
            inner_height,
            color_depth: 24,
            pixel_depth: 24,
            device_pixel_ratio,
        };

        self.current = Some(resolution.clone());
        resolution
    }

    /// Get the current resolution (generates one if not set)
    pub fn get_current(&mut self) -> ScreenResolution {
        if let Some(ref res) = self.current {
            res.clone()
        } else {
            self.generate()
        }
    }

    /// Generate JavaScript to override screen properties
    pub fn generate_js(&self, resolution: &ScreenResolution) -> String {
        format!(
            r#"
    // Screen resolution spoofing with consistency
    const screenProps = {{
        width: {screen_width},
        height: {screen_height},
        availWidth: {avail_width},
        availHeight: {avail_height},
        colorDepth: {color_depth},
        pixelDepth: {pixel_depth}
    }};

    Object.keys(screenProps).forEach(prop => {{
        originalDefineProperty(screen, prop, {{
            get: () => screenProps[prop],
            enumerable: true,
            configurable: true
        }});
    }});

    // Window size properties
    originalDefineProperty(window, 'outerWidth', {{
        get: () => {outer_width},
        enumerable: true,
        configurable: true
    }});

    originalDefineProperty(window, 'outerHeight', {{
        get: () => {outer_height},
        enumerable: true,
        configurable: true
    }});

    originalDefineProperty(window, 'innerWidth', {{
        get: () => {inner_width},
        enumerable: true,
        configurable: true
    }});

    originalDefineProperty(window, 'innerHeight', {{
        get: () => {inner_height},
        enumerable: true,
        configurable: true
    }});

    // Device pixel ratio
    originalDefineProperty(window, 'devicePixelRatio', {{
        get: () => {device_pixel_ratio},
        enumerable: true,
        configurable: true
    }});

    // Screen orientation
    if (screen.orientation) {{
        originalDefineProperty(screen.orientation, 'type', {{
            get: () => {screen_width} > {screen_height} ? 'landscape-primary' : 'portrait-primary',
            enumerable: true,
            configurable: true
        }});
    }}
"#,
            screen_width = resolution.screen_width,
            screen_height = resolution.screen_height,
            avail_width = resolution.avail_width,
            avail_height = resolution.avail_height,
            color_depth = resolution.color_depth,
            pixel_depth = resolution.pixel_depth,
            outer_width = resolution.outer_width,
            outer_height = resolution.outer_height,
            inner_width = resolution.inner_width,
            inner_height = resolution.inner_height,
            device_pixel_ratio = resolution.device_pixel_ratio,
        )
    }

    /// Validate that a resolution has consistent properties
    pub fn validate(resolution: &ScreenResolution) -> Result<(), String> {
        // Screen >= available
        if resolution.screen_width < resolution.avail_width {
            return Err("Available width cannot exceed screen width".to_string());
        }
        if resolution.screen_height < resolution.avail_height {
            return Err("Available height cannot exceed screen height".to_string());
        }

        // Available >= outer
        if resolution.avail_width < resolution.outer_width {
            return Err("Outer width cannot exceed available width".to_string());
        }
        if resolution.avail_height < resolution.outer_height {
            return Err("Outer height cannot exceed available height".to_string());
        }

        // Outer >= inner
        if resolution.outer_width < resolution.inner_width {
            return Err("Inner width cannot exceed outer width".to_string());
        }
        if resolution.outer_height < resolution.inner_height {
            return Err("Inner height cannot exceed outer height".to_string());
        }

        // Color depths must be valid
        if resolution.color_depth != 24 && resolution.color_depth != 32 {
            return Err(format!("Invalid color depth: {}", resolution.color_depth));
        }
        if resolution.pixel_depth != 24 && resolution.pixel_depth != 32 {
            return Err(format!("Invalid pixel depth: {}", resolution.pixel_depth));
        }

        // Device pixel ratio must be realistic
        let valid_ratios = [1.0, 1.25, 1.5, 2.0, 2.5, 3.0];
        if !valid_ratios.contains(&resolution.device_pixel_ratio) {
            return Err(format!(
                "Invalid device pixel ratio: {}",
                resolution.device_pixel_ratio
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolution_generation() {
        let mut manager = ScreenResolutionManager::new();
        let resolution = manager.generate();

        // Validate consistency
        assert!(ScreenResolutionManager::validate(&resolution).is_ok());

        // Check realistic values
        assert!(resolution.screen_width >= 1280);
        assert!(resolution.screen_height >= 720);
        assert!(resolution.color_depth == 24 || resolution.color_depth == 32);
        assert!(resolution.device_pixel_ratio >= 1.0 && resolution.device_pixel_ratio <= 3.0);
    }

    #[test]
    fn test_resolution_consistency() {
        let mut manager = ScreenResolutionManager::new();
        let resolution = manager.generate();

        // Screen >= Available
        assert!(resolution.screen_width >= resolution.avail_width);
        assert!(resolution.screen_height >= resolution.avail_height);

        // Available >= Outer
        assert!(resolution.avail_width >= resolution.outer_width);
        assert!(resolution.avail_height >= resolution.outer_height);

        // Outer >= Inner
        assert!(resolution.outer_width >= resolution.inner_width);
        assert!(resolution.outer_height >= resolution.inner_height);
    }

    #[test]
    fn test_js_generation() {
        let mut manager = ScreenResolutionManager::new();
        let resolution = manager.generate();
        let js = manager.generate_js(&resolution);

        assert!(js.contains("screenProps"));
        assert!(js.contains("outerWidth"));
        assert!(js.contains("innerWidth"));
        assert!(js.contains("devicePixelRatio"));
    }

    #[test]
    fn test_validation() {
        let valid_resolution = ScreenResolution {
            screen_width: 1920,
            screen_height: 1080,
            avail_width: 1920,
            avail_height: 1040,
            outer_width: 1920,
            outer_height: 1040,
            inner_width: 1920,
            inner_height: 920,
            color_depth: 24,
            pixel_depth: 24,
            device_pixel_ratio: 1.0,
        };

        assert!(ScreenResolutionManager::validate(&valid_resolution).is_ok());

        // Invalid: inner > outer
        let invalid_resolution = ScreenResolution {
            inner_width: 2000,
            outer_width: 1920,
            ..valid_resolution
        };

        assert!(ScreenResolutionManager::validate(&invalid_resolution).is_err());
    }
}
