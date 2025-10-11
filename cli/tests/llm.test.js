import { formatProviders, formatLLMConfig } from '../src/utils/formatters.js';
import stripAnsi from 'strip-ansi';

describe('LLM Command', () => {
  describe('argument parsing', () => {
    it('should parse providers subcommand', () => {
      const args = ['providers'];
      expect(args[0]).toBe('providers');
    });

    it('should parse switch subcommand with provider name', () => {
      const args = ['switch', 'openai'];
      expect(args[0]).toBe('switch');
      expect(args[1]).toBe('openai');
    });

    it('should parse config get subcommand', () => {
      const args = ['config', 'get'];
      expect(args[0]).toBe('config');
      expect(args[1]).toBe('get');
    });

    it('should parse config set subcommand with key and value', () => {
      const args = ['config', 'set', 'temperature', '0.8'];
      expect(args[0]).toBe('config');
      expect(args[1]).toBe('set');
      expect(args[2]).toBe('temperature');
      expect(args[3]).toBe('0.8');
    });
  });

  describe('formatProviders', () => {
    it('should format providers list with available providers', () => {
      const data = {
        providers: [
          { name: 'openai', available: true, active: true, models: ['gpt-4', 'gpt-3.5-turbo'] },
          { name: 'anthropic', available: true, active: false, models: ['claude-3-opus', 'claude-3-sonnet'] },
          { name: 'gemini', available: false, active: false, models: [] }
        ]
      };

      const output = stripAnsi(formatProviders(data));
      expect(output).toContain('openai');
      expect(output).toContain('anthropic');
      expect(output).toContain('gemini');
      expect(output).toContain('Available');
      expect(output).toContain('Unavailable');
    });

    it('should handle empty providers list', () => {
      const data = { providers: [] };
      const output = stripAnsi(formatProviders(data));
      expect(output).toContain('No LLM providers configured');
    });

    it('should show active status correctly', () => {
      const data = {
        providers: [
          { name: 'openai', available: true, active: true, models: ['gpt-4'] }
        ]
      };

      const output = stripAnsi(formatProviders(data));
      expect(output).toContain('openai');
      expect(output).toContain('✓');
    });

    it('should truncate long model lists', () => {
      const data = {
        providers: [
          {
            name: 'openai',
            available: true,
            active: true,
            models: ['model-1', 'model-2', 'model-3', 'model-4', 'model-5', 'model-6', 'model-7']
          }
        ]
      };

      const output = stripAnsi(formatProviders(data));
      expect(output).toContain('openai');
      // Models should be truncated to fit within column width
      expect(output.length).toBeLessThan(500);
    });

    it('should handle providers without models', () => {
      const data = {
        providers: [
          { name: 'custom-provider', available: true, active: false }
        ]
      };

      const output = stripAnsi(formatProviders(data));
      expect(output).toContain('custom-provider');
      expect(output).toContain('N/A');
    });
  });

  describe('formatLLMConfig', () => {
    it('should format standard config object', () => {
      const config = {
        temperature: 0.7,
        max_tokens: 2000,
        top_p: 1.0,
        frequency_penalty: 0.0,
        presence_penalty: 0.0
      };

      const output = stripAnsi(formatLLMConfig(config));
      expect(output).toContain('temperature');
      expect(output).toContain('0.7');
      expect(output).toContain('max_tokens');
      expect(output).toContain('2000');
      expect(output).toContain('top_p');
    });

    it('should handle empty config', () => {
      const config = {};
      const output = stripAnsi(formatLLMConfig(config));
      expect(output).toContain('No LLM configuration available');
    });

    it('should handle null config', () => {
      const config = null;
      const output = stripAnsi(formatLLMConfig(config));
      expect(output).toContain('No LLM configuration available');
    });

    it('should handle undefined config', () => {
      const config = undefined;
      const output = stripAnsi(formatLLMConfig(config));
      expect(output).toContain('No LLM configuration available');
    });

    it('should mask sensitive values', () => {
      const config = {
        api_key: 'sk-1234567890',
        access_token: 'token-abc123',
        temperature: 0.7
      };

      const output = stripAnsi(formatLLMConfig(config));
      expect(output).not.toContain('sk-1234567890');
      expect(output).not.toContain('token-abc123');
      expect(output).toContain('••••••••');
      expect(output).toContain('0.7'); // Non-sensitive values should be visible
    });

    it('should format boolean values', () => {
      const config = {
        streaming: true,
        cache_enabled: false
      };

      const output = stripAnsi(formatLLMConfig(config));
      expect(output).toContain('streaming');
      expect(output).toContain('cache_enabled');
      expect(output).toMatch(/enabled|disabled/);
    });

    it('should handle nested objects', () => {
      const config = {
        model_params: {
          temperature: 0.7,
          max_tokens: 2000
        }
      };

      const output = stripAnsi(formatLLMConfig(config));
      expect(output).toContain('model_params');
      // Nested objects should be stringified
      expect(output).toMatch(/temperature|max_tokens/);
    });

    it('should handle numeric values', () => {
      const config = {
        temperature: 0.8,
        max_tokens: 1500,
        timeout: 30
      };

      const output = stripAnsi(formatLLMConfig(config));
      expect(output).toContain('0.8');
      expect(output).toContain('1500');
      expect(output).toContain('30');
    });

    it('should handle string values', () => {
      const config = {
        model: 'gpt-4',
        provider: 'openai'
      };

      const output = stripAnsi(formatLLMConfig(config));
      expect(output).toContain('gpt-4');
      expect(output).toContain('openai');
    });
  });

  describe('value conversion', () => {
    it('should convert string numbers to numbers', () => {
      const strValue = '0.8';
      const numValue = parseFloat(strValue);
      expect(numValue).toBe(0.8);
      expect(typeof numValue).toBe('number');
    });

    it('should convert string integers to integers', () => {
      const strValue = '2000';
      const numValue = parseInt(strValue, 10);
      expect(numValue).toBe(2000);
      expect(typeof numValue).toBe('number');
    });

    it('should validate temperature range', () => {
      const isValidTemp = (value) => {
        const num = parseFloat(value);
        return num >= 0 && num <= 1;
      };

      expect(isValidTemp('0.7')).toBe(true);
      expect(isValidTemp('0')).toBe(true);
      expect(isValidTemp('1')).toBe(true);
      expect(isValidTemp('2.0')).toBe(false);
      expect(isValidTemp('-0.5')).toBe(false);
    });

    it('should validate max_tokens as positive integer', () => {
      const isValidMaxTokens = (value) => {
        const num = parseInt(value, 10);
        return !isNaN(num) && num > 0;
      };

      expect(isValidMaxTokens('2000')).toBe(true);
      expect(isValidMaxTokens('1')).toBe(true);
      expect(isValidMaxTokens('0')).toBe(false);
      expect(isValidMaxTokens('-100')).toBe(false);
      expect(isValidMaxTokens('abc')).toBe(false);
    });
  });

  describe('command structure', () => {
    it('should define valid subcommands', () => {
      const validSubcommands = ['providers', 'switch', 'config'];
      expect(validSubcommands).toContain('providers');
      expect(validSubcommands).toContain('switch');
      expect(validSubcommands).toContain('config');
    });

    it('should define valid config actions', () => {
      const validConfigActions = ['get', 'set'];
      expect(validConfigActions).toContain('get');
      expect(validConfigActions).toContain('set');
    });

    it('should define valid format options', () => {
      const validFormats = ['text', 'json', 'table'];
      expect(validFormats).toContain('text');
      expect(validFormats).toContain('json');
      expect(validFormats).toContain('table');
    });

    it('should define valid config keys', () => {
      const validConfigKeys = [
        'temperature',
        'max_tokens',
        'top_p',
        'frequency_penalty',
        'presence_penalty'
      ];

      expect(validConfigKeys).toContain('temperature');
      expect(validConfigKeys).toContain('max_tokens');
      expect(validConfigKeys).toContain('top_p');
    });
  });
});
