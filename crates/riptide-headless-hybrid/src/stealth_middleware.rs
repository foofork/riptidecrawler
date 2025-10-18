//! Stealth middleware to apply EventMesh stealth features to spider-chrome pages

use anyhow::{Context, Result};
use riptide_stealth::{FingerprintGenerator, StealthController};
use tracing::debug;

// Import chromiumoxide Page type (aligned with riptide-engine pattern)
use chromiumoxide::Page;

/// Stealth middleware for applying anti-detection measures
pub struct StealthMiddleware;

impl StealthMiddleware {
    /// Apply all stealth measures to a page
    pub async fn apply_all(page: &Page, controller: &mut StealthController) -> Result<()> {
        // Inject stealth JavaScript
        Self::inject_stealth_js(page, controller).await?;

        // Set realistic viewport
        Self::set_viewport(page).await?;

        // Override navigator properties
        Self::override_navigator(page).await?;

        // Apply fingerprinting protection
        Self::apply_fingerprinting_protection(page, controller).await?;

        debug!("All stealth measures applied successfully");
        Ok(())
    }

    /// Inject stealth JavaScript from controller
    async fn inject_stealth_js(page: &Page, controller: &mut StealthController) -> Result<()> {
        let stealth_js = controller.get_stealth_js();

        page.evaluate(stealth_js.as_str())
            .await
            .context("Failed to inject stealth JavaScript")?;

        debug!("Stealth JavaScript injected");
        Ok(())
    }

    /// Set realistic viewport dimensions
    async fn set_viewport(page: &Page) -> Result<()> {
        let viewport_script = r#"
            Object.defineProperty(window, 'innerWidth', {
                get: () => 1920
            });
            Object.defineProperty(window, 'innerHeight', {
                get: () => 1080
            });
            Object.defineProperty(window, 'outerWidth', {
                get: () => 1920
            });
            Object.defineProperty(window, 'outerHeight', {
                get: () => 1080
            });
        "#;

        page.evaluate(viewport_script)
            .await
            .context("Failed to set viewport")?;

        debug!("Viewport configured");
        Ok(())
    }

    /// Override navigator properties to avoid detection
    async fn override_navigator(page: &Page) -> Result<()> {
        let navigator_script = r#"
            // Override webdriver flag
            Object.defineProperty(navigator, 'webdriver', {
                get: () => undefined,
                configurable: true
            });

            // Mock plugins
            Object.defineProperty(navigator, 'plugins', {
                get: () => [{
                    name: 'Chrome PDF Plugin',
                    description: 'Portable Document Format',
                    filename: 'internal-pdf-viewer',
                    length: 1
                }, {
                    name: 'Chrome PDF Viewer',
                    description: 'Portable Document Format',
                    filename: 'mhjfbmdgcfjbbpaeojofohoefgiehjai',
                    length: 1
                }, {
                    name: 'Native Client',
                    description: 'Native Client Executable',
                    filename: 'internal-nacl-plugin',
                    length: 2
                }],
                configurable: true
            });

            // Set realistic languages
            Object.defineProperty(navigator, 'languages', {
                get: () => ['en-US', 'en'],
                configurable: true
            });

            // Override permissions
            const originalQuery = window.navigator.permissions.query;
            window.navigator.permissions.query = (parameters) => (
                parameters.name === 'notifications' ?
                    Promise.resolve({ state: Notification.permission }) :
                    originalQuery(parameters)
            );

            // Override chrome object
            window.chrome = {
                runtime: {},
                loadTimes: function() {},
                csi: function() {},
                app: {}
            };
        "#;

        page.evaluate(navigator_script)
            .await
            .context("Failed to override navigator")?;

        debug!("Navigator properties overridden");
        Ok(())
    }

    /// Apply fingerprinting protection measures
    async fn apply_fingerprinting_protection(
        page: &Page,
        _controller: &StealthController,
    ) -> Result<()> {
        // Generate realistic fingerprint
        let fingerprint = FingerprintGenerator::generate();

        let fingerprint_script = format!(
            r#"
            // Override WebGL parameters
            const getParameter = WebGLRenderingContext.prototype.getParameter;
            WebGLRenderingContext.prototype.getParameter = function(parameter) {{
                if (parameter === 37445) {{
                    return '{}';
                }}
                if (parameter === 37446) {{
                    return '{}';
                }}
                return getParameter.call(this, parameter);
            }};

            // Add canvas noise
            const toDataURL = HTMLCanvasElement.prototype.toDataURL;
            HTMLCanvasElement.prototype.toDataURL = function() {{
                const context = this.getContext('2d');
                if (context) {{
                    const imageData = context.getImageData(0, 0, this.width, this.height);
                    for (let i = 0; i < imageData.data.length; i += 4) {{
                        imageData.data[i] = imageData.data[i] + Math.random() * 0.5;
                    }}
                    context.putImageData(imageData, 0, 0);
                }}
                return toDataURL.apply(this, arguments);
            }};

            // Override audio context
            const AudioContext = window.AudioContext || window.webkitAudioContext;
            if (AudioContext) {{
                const getChannelData = AudioBuffer.prototype.getChannelData;
                AudioBuffer.prototype.getChannelData = function() {{
                    const originalData = getChannelData.apply(this, arguments);
                    for (let i = 0; i < originalData.length; i++) {{
                        originalData[i] = originalData[i] + Math.random() * 0.0001;
                    }}
                    return originalData;
                }};
            }}

            // Override hardware concurrency
            Object.defineProperty(navigator, 'hardwareConcurrency', {{
                get: () => {},
                configurable: true
            }});

            // Override device memory
            Object.defineProperty(navigator, 'deviceMemory', {{
                get: () => {},
                configurable: true
            }});

            // Override screen properties
            Object.defineProperty(screen, 'width', {{
                get: () => {},
                configurable: true
            }});
            Object.defineProperty(screen, 'height', {{
                get: () => {},
                configurable: true
            }});
            Object.defineProperty(screen, 'colorDepth', {{
                get: () => {},
                configurable: true
            }});
            "#,
            fingerprint.webgl_vendor,
            fingerprint.webgl_renderer,
            fingerprint.hardware_concurrency,
            fingerprint.device_memory,
            fingerprint.screen_resolution.0,
            fingerprint.screen_resolution.1,
            fingerprint.color_depth,
        );

        page.evaluate(fingerprint_script.as_str())
            .await
            .context("Failed to apply fingerprinting protection")?;

        debug!("Fingerprinting protection applied");
        Ok(())
    }
}

/// Convenience function to apply stealth to a page
pub async fn apply_stealth(page: &Page, controller: &mut StealthController) -> Result<()> {
    StealthMiddleware::apply_all(page, controller)
        .await
        .context("Failed to apply stealth measures")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use riptide_stealth::StealthPreset;

    #[tokio::test]
    async fn test_stealth_middleware_creation() {
        let mut controller = StealthController::from_preset(StealthPreset::Medium);
        assert!(controller.get_stealth_js().len() > 0);
    }
}
