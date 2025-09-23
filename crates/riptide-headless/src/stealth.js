// Stealth JavaScript injection - injected early in page lifecycle
// Override webdriver detection
Object.defineProperty(navigator, 'webdriver', {
  get: () => false,
  enumerable: true,
  configurable: true
});

// Override navigator.plugins to appear natural
Object.defineProperty(navigator, 'plugins', {
  get: () => [
    {name: 'Chrome PDF Plugin', filename: 'internal-pdf-viewer'},
    {name: 'Chrome PDF Viewer', filename: 'mhjfbmdgcfjbbpaeojofohoefgiehjai'},
    {name: 'Native Client', filename: 'internal-nacl-plugin'}
  ],
  enumerable: true,
  configurable: true
});

// Override languages to appear natural
Object.defineProperty(navigator, 'languages', {
  get: () => ['en-US', 'en'],
  enumerable: true,
  configurable: true
});

// Hide Chrome automation
if (window.chrome) {
  window.chrome.runtime = {
    id: 'aabbccddeeffgghhiijjkkllmmnnooppqq',
    onMessage: {}
  };
}

// Canvas fingerprint noise injection
const originalToDataURL = HTMLCanvasElement.prototype.toDataURL;
HTMLCanvasElement.prototype.toDataURL = function(...args) {
  const context = this.getContext('2d');
  if (context) {
    const imageData = context.getImageData(0, 0, this.width, this.height);
    // Add tiny noise to break fingerprinting
    for (let i = 0; i < imageData.data.length; i += 4) {
      imageData.data[i] += (Math.random() - 0.5) * 0.1;
      imageData.data[i+1] += (Math.random() - 0.5) * 0.1;
      imageData.data[i+2] += (Math.random() - 0.5) * 0.1;
    }
    context.putImageData(imageData, 0, 0);
  }
  return originalToDataURL.apply(this, args);
};

// Enhanced WebGL vendor spoofing with noise injection
const getParameter = WebGLRenderingContext.prototype.getParameter;
WebGLRenderingContext.prototype.getParameter = function(parameter) {
  // Common realistic GPU combinations
  const gpuConfigs = [
    { vendor: 'Intel Inc.', renderer: 'Intel Iris OpenGL Engine' },
    { vendor: 'NVIDIA Corporation', renderer: 'NVIDIA GeForce GTX 1060/PCIe/SSE2' },
    { vendor: 'ATI Technologies Inc.', renderer: 'AMD Radeon RX 580' },
    { vendor: 'Intel Inc.', renderer: 'Intel UHD Graphics 630' }
  ];

  const selectedConfig = gpuConfigs[Math.floor(Math.random() * gpuConfigs.length)];

  if (parameter === 37445) return selectedConfig.vendor; // UNMASKED_VENDOR_WEBGL
  if (parameter === 37446) return selectedConfig.renderer; // UNMASKED_RENDERER_WEBGL

  // Add noise to other WebGL parameters
  const result = getParameter.apply(this, arguments);
  if (typeof result === 'number' && parameter !== 36347 && parameter !== 36348) {
    // Add small random variation to numeric parameters (except width/height)
    return result + (Math.random() - 0.5) * 0.001;
  }

  return result;
};

// Override WebGL2 context as well
if (window.WebGL2RenderingContext) {
  const getParameter2 = WebGL2RenderingContext.prototype.getParameter;
  WebGL2RenderingContext.prototype.getParameter = WebGLRenderingContext.prototype.getParameter;
}

// Override getUserMedia to avoid detection
if (navigator.mediaDevices && navigator.mediaDevices.getUserMedia) {
  const originalGetUserMedia = navigator.mediaDevices.getUserMedia;
  navigator.mediaDevices.getUserMedia = function(...args) {
    return originalGetUserMedia.apply(this, args).catch(() => {
      throw new DOMException('Permission denied', 'NotAllowedError');
    });
  };
}

// Permissions API override
if (navigator.permissions) {
  const originalQuery = navigator.permissions.query;
  navigator.permissions.query = async (parameters) => {
    if (parameters.name === 'notifications') {
      return Promise.resolve({ state: 'default' });
    }
    return originalQuery.apply(navigator.permissions, arguments);
  };
}

// Hide automation-specific properties
delete navigator.__webdriver_evaluate;
delete navigator.__webdriver_script_func;
delete navigator.__webdriver_script_fn;
delete navigator.__fxdriver_evaluate;
delete navigator.__fxdriver_unwrapped;
delete navigator.__driver_evaluate;
delete navigator.__webdriver_unwrapped;
delete navigator.__driver_unwrapped;

// Remove automation flags from window
delete window.__nightmare;
delete window._phantom;
delete window.Buffer;
delete window.emit;
delete window.spawn;

// Override Date and timezone detection
const originalDate = Date;
Date = class extends originalDate {
  getTimezoneOffset() {
    // Return timezone offset based on spoofed locale
    const timezones = {
      'en-US': 300, // EST
      'en-GB': 0,   // GMT
      'de-DE': -60, // CET
      'fr-FR': -60, // CET
      'ja-JP': -540 // JST
    };
    const locale = navigator.language || 'en-US';
    return timezones[locale] || 300;
  }

  toLocaleString(...args) {
    // Override to return consistent format
    return super.toLocaleString('en-US', ...args);
  }
};

// Override Intl.DateTimeFormat for timezone consistency
if (window.Intl && window.Intl.DateTimeFormat) {
  const originalDateTimeFormat = window.Intl.DateTimeFormat;
  window.Intl.DateTimeFormat = function(...args) {
    if (args.length === 0 || !args[0]) {
      args[0] = 'en-US';
    }
    return new originalDateTimeFormat(...args);
  };
}

// Add realistic screen properties
Object.defineProperty(screen, 'availWidth', {
  get: () => screen.width,
});

Object.defineProperty(screen, 'availHeight', {
  get: () => screen.height - 40, // Account for taskbar
});

// Spoof battery API
if (navigator.getBattery) {
  navigator.getBattery = () => Promise.resolve({
    charging: true,
    chargingTime: 0,
    dischargingTime: Infinity,
    level: 1,
    addEventListener: () => {},
    removeEventListener: () => {},
    dispatchEvent: () => true
  });
}

// Hide headless indicators in document
Object.defineProperty(document, 'hidden', {
  get: () => false,
});

Object.defineProperty(document, 'visibilityState', {
  get: () => 'visible',
});

// Override console methods to avoid detection
const originalConsoleDebug = console.debug;
console.debug = function(...args) {
  // Filter out Puppeteer/Playwright debug messages
  const message = args.join(' ');
  if (!message.includes('DevTools') && !message.includes('Runtime.consoleAPICalled')) {
    originalConsoleDebug.apply(this, args);
  }
};

// WebRTC leak prevention
if (window.RTCPeerConnection) {
  const originalRTCPeerConnection = window.RTCPeerConnection;
  window.RTCPeerConnection = function(...args) {
    const pc = new originalRTCPeerConnection(...args);

    // Override createDataChannel to prevent fingerprinting
    const originalCreateDataChannel = pc.createDataChannel;
    pc.createDataChannel = function(...args) {
      return originalCreateDataChannel.apply(this, args);
    };

    return pc;
  };
}

// Disable WebRTC IP leak
if (navigator.mediaDevices && navigator.mediaDevices.enumerateDevices) {
  const originalEnumerateDevices = navigator.mediaDevices.enumerateDevices;
  navigator.mediaDevices.enumerateDevices = function() {
    return Promise.resolve([
      { deviceId: 'default', kind: 'audioinput', label: 'Default - Microphone' },
      { deviceId: 'default', kind: 'audiooutput', label: 'Default - Speaker' }
    ]);
  };
}

// Enhanced language and locale spoofing
Object.defineProperty(navigator, 'language', {
  get: () => {
    const languages = ['en-US', 'en-GB', 'de-DE', 'fr-FR'];
    return languages[Math.floor(Math.random() * languages.length)];
  },
  enumerable: true,
  configurable: true
});

// Memory info spoofing
if (window.performance && window.performance.memory) {
  Object.defineProperty(window.performance, 'memory', {
    get: () => ({
      usedJSHeapSize: 16777216 + Math.random() * 8388608,
      totalJSHeapSize: 33554432 + Math.random() * 16777216,
      jsHeapSizeLimit: 2147483648
    }),
    enumerable: true,
    configurable: true
  });
}

// Audio context fingerprinting protection
if (window.AudioContext || window.webkitAudioContext) {
  const AudioContextClass = window.AudioContext || window.webkitAudioContext;
  const originalAudioContext = AudioContextClass.prototype.createAnalyser;

  AudioContextClass.prototype.createAnalyser = function() {
    const analyser = originalAudioContext.apply(this, arguments);
    const originalGetFloatFrequencyData = analyser.getFloatFrequencyData;

    analyser.getFloatFrequencyData = function(array) {
      originalGetFloatFrequencyData.apply(this, arguments);
      // Add noise to audio fingerprinting
      for (let i = 0; i < array.length; i++) {
        array[i] += (Math.random() - 0.5) * 0.001;
      }
    };

    return analyser;
  };
}

// Hardware concurrency spoofing
Object.defineProperty(navigator, 'hardwareConcurrency', {
  get: () => {
    const cores = [2, 4, 6, 8, 12, 16];
    return cores[Math.floor(Math.random() * cores.length)];
  },
  enumerable: true,
  configurable: true
});

// Device memory spoofing
if (navigator.deviceMemory !== undefined) {
  Object.defineProperty(navigator, 'deviceMemory', {
    get: () => {
      const memories = [2, 4, 8, 16];
      return memories[Math.floor(Math.random() * memories.length)];
    },
    enumerable: true,
    configurable: true
  });
}