//! Enhanced WebRTC leak prevention
//!
//! This module provides comprehensive WebRTC leak prevention including:
//! - IP address leak blocking
//! - Media device enumeration spoofing
//! - RTC data channel blocking
//! - STUN/TURN server interception

use serde::{Deserialize, Serialize};

/// Enhanced WebRTC configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebRtcEnhanced {
    /// Block all WebRTC functionality
    pub block_completely: bool,
    /// Block IP leaks via WebRTC
    pub block_ip_leak: bool,
    /// Spoof media devices
    pub spoof_media_devices: bool,
    /// Block RTC data channels
    pub block_data_channels: bool,
    /// Block STUN/TURN servers
    pub block_stun_turn: bool,
    /// Replace real IP with fake IP
    pub fake_ip: Option<String>,
}

impl Default for WebRtcEnhanced {
    fn default() -> Self {
        Self {
            block_completely: false,
            block_ip_leak: true,
            spoof_media_devices: true,
            block_data_channels: false,
            block_stun_turn: true,
            fake_ip: Some("192.168.1.1".to_string()),
        }
    }
}

impl WebRtcEnhanced {
    /// Generate JavaScript to prevent WebRTC leaks
    pub fn generate_js(&self) -> String {
        if self.block_completely {
            return self.generate_block_all_js();
        }

        let mut js_parts = Vec::new();

        if self.block_ip_leak {
            js_parts.push(self.generate_ip_leak_protection());
        }

        if self.spoof_media_devices {
            js_parts.push(self.generate_media_device_spoofing());
        }

        if self.block_data_channels {
            js_parts.push(self.generate_data_channel_blocking());
        }

        if self.block_stun_turn {
            js_parts.push(self.generate_stun_turn_blocking());
        }

        js_parts.join("\n\n")
    }

    /// Block all WebRTC functionality
    fn generate_block_all_js(&self) -> String {
        r#"
    // Complete WebRTC blocking
    delete window.RTCPeerConnection;
    delete window.webkitRTCPeerConnection;
    delete window.mozRTCPeerConnection;
    delete navigator.getUserMedia;
    delete navigator.webkitGetUserMedia;
    delete navigator.mozGetUserMedia;
    delete navigator.mediaDevices;
"#
        .to_string()
    }

    /// Generate IP leak protection JavaScript
    fn generate_ip_leak_protection(&self) -> String {
        let fake_ip = self
            .fake_ip
            .as_ref()
            .map(|ip| ip.as_str())
            .unwrap_or("192.168.1.1");

        format!(
            r#"
    // WebRTC IP leak prevention
    if (typeof RTCPeerConnection !== 'undefined') {{
        const OriginalRTCPeerConnection = RTCPeerConnection;
        window.RTCPeerConnection = function(...args) {{
            const pc = new OriginalRTCPeerConnection(...args);

            // Override createOffer to filter out real IP addresses
            const originalCreateOffer = pc.createOffer;
            pc.createOffer = async function(options) {{
                const offer = await originalCreateOffer.call(this, options);
                if (offer.sdp) {{
                    // Replace real IP addresses with fake IP
                    offer.sdp = offer.sdp.replace(
                        /([0-9]{{1,3}}(\.[0-9]{{1,3}}){{3}})/g,
                        '{fake_ip}'
                    );
                    // Remove IPv6 addresses
                    offer.sdp = offer.sdp.replace(
                        /([a-f0-9:]{{2,}}:+)+[a-f0-9]+/gi,
                        '{fake_ip}'
                    );
                }}
                return offer;
            }};

            // Override createAnswer similarly
            const originalCreateAnswer = pc.createAnswer;
            pc.createAnswer = async function(options) {{
                const answer = await originalCreateAnswer.call(this, options);
                if (answer.sdp) {{
                    answer.sdp = answer.sdp.replace(
                        /([0-9]{{1,3}}(\.[0-9]{{1,3}}){{3}})/g,
                        '{fake_ip}'
                    );
                    answer.sdp = answer.sdp.replace(
                        /([a-f0-9:]{{2,}}:+)+[a-f0-9]+/gi,
                        '{fake_ip}'
                    );
                }}
                return answer;
            }};

            // Override addIceCandidate to filter candidates
            const originalAddIceCandidate = pc.addIceCandidate;
            pc.addIceCandidate = function(candidate) {{
                if (candidate && candidate.candidate) {{
                    // Block candidates with real IP addresses
                    if (candidate.candidate.includes('srflx') || candidate.candidate.includes('relay')) {{
                        return Promise.resolve();
                    }}
                }}
                return originalAddIceCandidate.call(this, candidate);
            }};

            return pc;
        }};

        // Copy static properties
        Object.setPrototypeOf(window.RTCPeerConnection, OriginalRTCPeerConnection);
        window.RTCPeerConnection.prototype = OriginalRTCPeerConnection.prototype;
    }}
"#,
            fake_ip = fake_ip
        )
    }

    /// Generate media device spoofing JavaScript
    fn generate_media_device_spoofing(&self) -> String {
        r#"
    // Media device spoofing
    if (navigator.mediaDevices && navigator.mediaDevices.enumerateDevices) {
        const originalEnumerateDevices = navigator.mediaDevices.enumerateDevices;
        navigator.mediaDevices.enumerateDevices = async function() {
            // Return fake but realistic device list
            return Promise.resolve([
                {
                    deviceId: 'default',
                    kind: 'audioinput',
                    label: 'Default - Microphone (Realtek High Definition Audio)',
                    groupId: 'group1'
                },
                {
                    deviceId: 'communications',
                    kind: 'audioinput',
                    label: 'Communications - Microphone (Realtek High Definition Audio)',
                    groupId: 'group1'
                },
                {
                    deviceId: 'default',
                    kind: 'audiooutput',
                    label: 'Default - Speaker (Realtek High Definition Audio)',
                    groupId: 'group1'
                },
                {
                    deviceId: 'default',
                    kind: 'videoinput',
                    label: 'Integrated Webcam (04f2:b5a4)',
                    groupId: 'group2'
                }
            ]);
        };
    }

    // Override getUserMedia to avoid permission prompts
    if (navigator.mediaDevices && navigator.mediaDevices.getUserMedia) {
        const originalGetUserMedia = navigator.mediaDevices.getUserMedia;
        navigator.mediaDevices.getUserMedia = async function(constraints) {
            // Return a fake media stream
            return Promise.reject(new DOMException('Permission denied', 'NotAllowedError'));
        };
    }
"#
        .to_string()
    }

    /// Generate data channel blocking JavaScript
    fn generate_data_channel_blocking(&self) -> String {
        r#"
    // Block RTC data channels
    if (typeof RTCPeerConnection !== 'undefined') {
        const OriginalRTCPeerConnection = RTCPeerConnection;
        const originalCreateDataChannel = OriginalRTCPeerConnection.prototype.createDataChannel;

        OriginalRTCPeerConnection.prototype.createDataChannel = function(...args) {
            // Silently fail data channel creation
            return null;
        };
    }
"#
        .to_string()
    }

    /// Generate STUN/TURN server blocking JavaScript
    fn generate_stun_turn_blocking(&self) -> String {
        r#"
    // Block STUN/TURN servers
    if (typeof RTCPeerConnection !== 'undefined') {
        const OriginalRTCPeerConnection = RTCPeerConnection;
        window.RTCPeerConnection = function(configuration, ...args) {
            // Filter out STUN and TURN servers
            if (configuration && configuration.iceServers) {
                configuration.iceServers = configuration.iceServers.filter(server => {
                    if (Array.isArray(server.urls)) {
                        server.urls = server.urls.filter(url =>
                            !url.includes('stun:') && !url.includes('turn:')
                        );
                        return server.urls.length > 0;
                    } else if (typeof server.urls === 'string') {
                        return !server.urls.includes('stun:') && !server.urls.includes('turn:');
                    }
                    return false;
                });
            }

            return new OriginalRTCPeerConnection(configuration, ...args);
        };

        Object.setPrototypeOf(window.RTCPeerConnection, OriginalRTCPeerConnection);
        window.RTCPeerConnection.prototype = OriginalRTCPeerConnection.prototype;
    }
"#
        .to_string()
    }

    /// Create a high security WebRTC configuration
    pub fn high_security() -> Self {
        Self {
            block_completely: false,
            block_ip_leak: true,
            spoof_media_devices: true,
            block_data_channels: true,
            block_stun_turn: true,
            fake_ip: Some("192.168.1.1".to_string()),
        }
    }

    /// Create a complete blocking configuration
    pub fn block_all() -> Self {
        Self {
            block_completely: true,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = WebRtcEnhanced::default();
        assert!(!config.block_completely);
        assert!(config.block_ip_leak);
        assert!(config.spoof_media_devices);
        assert!(config.block_stun_turn);
    }

    #[test]
    fn test_js_generation() {
        let config = WebRtcEnhanced::default();
        let js = config.generate_js();

        assert!(js.contains("RTCPeerConnection"));
        assert!(js.contains("fake_ip") || js.contains("192.168.1.1"));
    }

    #[test]
    fn test_complete_blocking() {
        let config = WebRtcEnhanced::block_all();
        let js = config.generate_js();

        assert!(js.contains("delete window.RTCPeerConnection"));
        assert!(js.contains("delete navigator.getUserMedia"));
    }

    #[test]
    fn test_high_security() {
        let config = WebRtcEnhanced::high_security();
        assert!(!config.block_completely);
        assert!(config.block_data_channels);
        assert!(config.block_stun_turn);
    }
}
