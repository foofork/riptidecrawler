/**
 * Configuration management
 * Stores user preferences in ~/.config/riptide-cli/
 */

import Conf from 'conf';

const schema = {
  'api-url': {
    type: 'string',
    default: 'http://localhost:8080'
  },
  'api-key': {
    type: 'string',
    default: ''
  },
  'default-concurrency': {
    type: 'number',
    default: 3
  },
  'default-cache-mode': {
    type: 'string',
    default: 'auto'
  },
  'default-format': {
    type: 'string',
    default: 'text'
  },
  'color-output': {
    type: 'boolean',
    default: true
  }
};

let configInstance = null;

export function getConfig() {
  if (!configInstance) {
    configInstance = new Conf({
      projectName: 'riptide-cli',
      schema
    });
  }
  return configInstance;
}

export function resetConfig() {
  const config = getConfig();
  config.clear();
  return config;
}
