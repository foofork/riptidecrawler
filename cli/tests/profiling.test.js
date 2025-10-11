import { jest } from '@jest/globals';
import { profilingCommand } from '../src/commands/profiling.js';
import { RipTideClient } from '../src/utils/api-client.js';
import fs from 'fs';

describe('Profiling Command', () => {
  let consoleLogSpy;
  let consoleErrorSpy;
  let processExitSpy;

  beforeEach(() => {
    // Spy on console methods
    consoleLogSpy = jest.spyOn(console, 'log').mockImplementation();
    consoleErrorSpy = jest.spyOn(console, 'error').mockImplementation();
    processExitSpy = jest.spyOn(process, 'exit').mockImplementation();

    // Mock RipTideClient methods
    jest.spyOn(RipTideClient.prototype, 'getMemoryProfile').mockResolvedValue({
      total: 1073741824,
      used: 536870912,
      free: 536870912,
      usage_percent: 50.0
    });

    jest.spyOn(RipTideClient.prototype, 'getCPUProfile').mockResolvedValue({
      usage_percent: 45.2,
      cores: 8,
      load_average: [1.5, 1.3, 1.1]
    });

    jest.spyOn(RipTideClient.prototype, 'getBottlenecks').mockResolvedValue({
      bottlenecks: [
        { component: 'database', severity: 'high', description: 'Slow queries' },
        { component: 'network', severity: 'medium', description: 'High latency' }
      ]
    });

    jest.spyOn(RipTideClient.prototype, 'getAllocations').mockResolvedValue({
      total_allocations: 1024,
      by_type: {
        string: 512,
        array: 256,
        object: 256
      }
    });

    jest.spyOn(RipTideClient.prototype, 'detectLeaks').mockResolvedValue({
      detected: true,
      potential_leaks: [
        { location: 'module/file.js:42', size: 1024000, description: 'Unreleased buffer' }
      ]
    });

    jest.spyOn(RipTideClient.prototype, 'createSnapshot').mockResolvedValue({
      snapshot_id: 'snapshot-123',
      timestamp: '2025-10-10T12:00:00Z',
      size_bytes: 1048576,
      memory: { used: 536870912 },
      cpu: { usage_percent: 45.2 }
    });

    // Mock fs.writeFileSync
    jest.spyOn(fs, 'writeFileSync').mockImplementation();
  });

  afterEach(() => {
    consoleLogSpy.mockRestore();
    consoleErrorSpy.mockRestore();
    processExitSpy.mockRestore();
    jest.restoreAllMocks();
  });

  describe('missing subcommand', () => {
    it('should show error when no subcommand provided', async () => {
      const mockCommand = {
        parent: { opts: () => ({}) }
      };

      await profilingCommand(null, {}, mockCommand);

      expect(consoleLogSpy).toHaveBeenCalledWith(expect.stringContaining('Profiling subcommand required'));
      expect(processExitSpy).toHaveBeenCalledWith(1);
    });
  });

  describe('memory subcommand', () => {
    it('should fetch and display memory profile', async () => {
      const mockCommand = {
        parent: { opts: () => ({}) }
      };

      await profilingCommand('memory', {}, mockCommand);

      expect(RipTideClient.prototype.getMemoryProfile).toHaveBeenCalled();
      expect(consoleLogSpy).toHaveBeenCalled();
    });

    it('should support JSON format output', async () => {
      const mockCommand = {
        parent: { opts: () => ({ json: true }) }
      };

      await profilingCommand('memory', {}, mockCommand);

      expect(RipTideClient.prototype.getMemoryProfile).toHaveBeenCalled();
      expect(consoleLogSpy).toHaveBeenCalledWith(expect.stringContaining('{'));
    });

    it('should write output to file when --output specified', async () => {
      const mockCommand = {
        parent: { opts: () => ({}) }
      };

      await profilingCommand('memory', { output: 'memory-profile.json' }, mockCommand);

      expect(fs.writeFileSync).toHaveBeenCalledWith(
        'memory-profile.json',
        expect.any(String)
      );
      expect(consoleLogSpy).toHaveBeenCalledWith(expect.stringContaining('Saved to memory-profile.json'));
    });
  });

  describe('cpu subcommand', () => {
    it('should fetch and display CPU profile', async () => {
      const mockCommand = {
        parent: { opts: () => ({}) }
      };

      await profilingCommand('cpu', {}, mockCommand);

      expect(RipTideClient.prototype.getCPUProfile).toHaveBeenCalled();
      expect(consoleLogSpy).toHaveBeenCalled();
    });

    it('should format CPU profile as JSON when requested', async () => {
      const mockCommand = {
        parent: { opts: () => ({}) }
      };

      await profilingCommand('cpu', { format: 'json' }, mockCommand);

      expect(RipTideClient.prototype.getCPUProfile).toHaveBeenCalled();
      expect(consoleLogSpy).toHaveBeenCalledWith(expect.stringContaining('{'));
    });
  });

  describe('bottlenecks subcommand', () => {
    it('should identify and display performance bottlenecks', async () => {
      const mockCommand = {
        parent: { opts: () => ({}) }
      };

      await profilingCommand('bottlenecks', {}, mockCommand);

      expect(RipTideClient.prototype.getBottlenecks).toHaveBeenCalled();
      expect(consoleLogSpy).toHaveBeenCalled();
    });

    it('should handle empty bottlenecks list', async () => {
      RipTideClient.prototype.getBottlenecks.mockResolvedValueOnce({
        bottlenecks: [],
        hotspots: []
      });

      const mockCommand = {
        parent: { opts: () => ({}) }
      };

      await profilingCommand('bottlenecks', {}, mockCommand);

      expect(RipTideClient.prototype.getBottlenecks).toHaveBeenCalled();
      expect(consoleLogSpy).toHaveBeenCalled();
    });
  });

  describe('allocations subcommand', () => {
    it('should fetch and display memory allocations', async () => {
      const mockCommand = {
        parent: { opts: () => ({}) }
      };

      await profilingCommand('allocations', {}, mockCommand);

      expect(RipTideClient.prototype.getAllocations).toHaveBeenCalled();
      expect(consoleLogSpy).toHaveBeenCalled();
    });

    it('should export allocations to JSON file', async () => {
      const mockCommand = {
        parent: { opts: () => ({}) }
      };

      await profilingCommand('allocations', {
        format: 'json',
        output: 'allocations.json'
      }, mockCommand);

      expect(fs.writeFileSync).toHaveBeenCalledWith(
        'allocations.json',
        expect.any(String)
      );
    });
  });

  describe('leaks subcommand', () => {
    it('should detect and report memory leaks', async () => {
      const mockCommand = {
        parent: { opts: () => ({}) }
      };

      await profilingCommand('leaks', {}, mockCommand);

      expect(RipTideClient.prototype.detectLeaks).toHaveBeenCalled();
      expect(consoleLogSpy).toHaveBeenCalled();
    });

    it('should report when no leaks detected', async () => {
      RipTideClient.prototype.detectLeaks.mockResolvedValueOnce({
        detected: false,
        potential_leaks: []
      });

      const mockCommand = {
        parent: { opts: () => ({}) }
      };

      await profilingCommand('leaks', {}, mockCommand);

      expect(RipTideClient.prototype.detectLeaks).toHaveBeenCalled();
      expect(consoleLogSpy).toHaveBeenCalled();
    });
  });

  describe('snapshot subcommand', () => {
    it('should create and display profiling snapshot', async () => {
      const mockCommand = {
        parent: { opts: () => ({}) }
      };

      await profilingCommand('snapshot', {}, mockCommand);

      expect(RipTideClient.prototype.createSnapshot).toHaveBeenCalled();
      expect(consoleLogSpy).toHaveBeenCalled();
    });

    it('should save snapshot to file', async () => {
      const mockCommand = {
        parent: { opts: () => ({}) }
      };

      await profilingCommand('snapshot', { output: 'snapshot.json' }, mockCommand);

      expect(fs.writeFileSync).toHaveBeenCalledWith(
        'snapshot.json',
        expect.any(String)
      );
      expect(consoleLogSpy).toHaveBeenCalledWith(expect.stringContaining('Saved to snapshot.json'));
    });
  });

  describe('error handling', () => {
    it('should handle API errors gracefully', async () => {
      const error = new Error('API connection failed');
      RipTideClient.prototype.getMemoryProfile.mockRejectedValueOnce(error);

      const mockCommand = {
        parent: { opts: () => ({}) }
      };

      await profilingCommand('memory', {}, mockCommand);

      expect(consoleLogSpy).toHaveBeenCalledWith(
        expect.stringContaining('Profiling failed')
      );
      expect(processExitSpy).toHaveBeenCalledWith(1);
    });

    it('should handle file write errors', async () => {
      fs.writeFileSync.mockImplementationOnce(() => {
        throw new Error('Permission denied');
      });

      const mockCommand = {
        parent: { opts: () => ({}) }
      };

      await profilingCommand('memory', { output: 'invalid/path/file.json' }, mockCommand);

      expect(consoleLogSpy).toHaveBeenCalledWith(
        expect.stringContaining('Profiling failed')
      );
      expect(processExitSpy).toHaveBeenCalledWith(1);
    });

    it('should handle invalid subcommands', async () => {
      const mockCommand = {
        parent: { opts: () => ({}) }
      };

      await profilingCommand('invalid-subcommand', {}, mockCommand);

      expect(consoleLogSpy).toHaveBeenCalledWith(
        expect.stringContaining('Unknown subcommand')
      );
      expect(processExitSpy).toHaveBeenCalledWith(1);
    });
  });

  describe('global options', () => {
    it('should respect global --json option', async () => {
      const mockCommand = {
        parent: { opts: () => ({ json: true }) }
      };

      await profilingCommand('memory', {}, mockCommand);

      expect(RipTideClient.prototype.getMemoryProfile).toHaveBeenCalled();
      expect(consoleLogSpy).toHaveBeenCalledWith(expect.stringContaining('{'));
    });

    it('should use custom API URL from global options', async () => {
      const mockCommand = {
        parent: { opts: () => ({ url: 'http://custom-api:8080' }) }
      };

      await profilingCommand('memory', {}, mockCommand);

      expect(RipTideClient.prototype.getMemoryProfile).toHaveBeenCalled();
    });

    it('should use API key from global options', async () => {
      const mockCommand = {
        parent: { opts: () => ({ apiKey: 'test-api-key-123' }) }
      };

      await profilingCommand('cpu', {}, mockCommand);

      expect(RipTideClient.prototype.getCPUProfile).toHaveBeenCalled();
    });
  });

  describe('output formatting', () => {
    it('should display recommendations for leak detection', async () => {
      const mockCommand = {
        parent: { opts: () => ({}) }
      };

      await profilingCommand('leaks', {}, mockCommand);

      expect(consoleLogSpy).toHaveBeenCalledWith(expect.stringContaining('Recommendations'));
    });

    it('should display recommendations for bottlenecks', async () => {
      RipTideClient.prototype.getBottlenecks.mockResolvedValueOnce({
        bottlenecks: [
          { component: 'database', severity: 'high', description: 'Slow queries' }
        ],
        hotspots: [
          { function: 'processData', time_ms: 1500 }
        ]
      });

      const mockCommand = {
        parent: { opts: () => ({}) }
      };

      await profilingCommand('bottlenecks', {}, mockCommand);

      expect(consoleLogSpy).toHaveBeenCalledWith(expect.stringContaining('Recommendations'));
    });

    it('should show snapshot details', async () => {
      const mockCommand = {
        parent: { opts: () => ({}) }
      };

      await profilingCommand('snapshot', {}, mockCommand);

      expect(consoleLogSpy).toHaveBeenCalledWith(expect.stringContaining('Next steps'));
      expect(consoleLogSpy).toHaveBeenCalledWith(expect.stringContaining('snapshot-123'));
    });

    it('should display warnings if present', async () => {
      RipTideClient.prototype.getMemoryProfile.mockResolvedValueOnce({
        total: 1073741824,
        used: 536870912,
        free: 536870912,
        usage_percent: 50.0,
        warnings: ['Memory usage is high', 'GC frequency increased']
      });

      const mockCommand = {
        parent: { opts: () => ({}) }
      };

      await profilingCommand('memory', {}, mockCommand);

      expect(consoleLogSpy).toHaveBeenCalledWith(expect.stringContaining('Warnings'));
    });
  });
});
