/**
 * Health command implementation with tiered health checks (QW-2)
 *
 * Implements three tiers of health checks:
 * - Fast check (2s): Liveness check for basic availability
 * - Full check (15s): Detailed health diagnostics with all components
 * - On-error check (500ms): Immediate verification after errors
 */

import chalk from 'chalk';
import ora from 'ora';
import { RipTideClient } from '../utils/api-client.js';
import { formatHealth, formatJSON } from '../utils/formatters.js';

export async function healthCommand(options, command) {
  const globalOpts = command.parent.opts();

  if (options.watch) {
    return watchHealth(globalOpts, options);
  }

  // QW-2: Determine health check mode
  const mode = options.mode || 'fast';
  const timeoutMs = getTimeoutForMode(mode);
  const spinner = ora(`Checking health (${mode} mode, ${timeoutMs}ms)...`).start();

  try {
    const client = new RipTideClient({
      url: globalOpts.url,
      apiKey: globalOpts.apiKey
    });

    const health = await performHealthCheck(client, mode, timeoutMs);

    spinner.stop();

    if (globalOpts.json) {
      console.log(formatJSON(health));
    } else {
      console.log(formatHealth(health));
      console.log(chalk.gray(`\nHealth check mode: ${mode} (${timeoutMs}ms timeout)`));
    }

    // Exit code based on health
    if (health.status !== 'healthy') {
      process.exit(1);
    }

  } catch (error) {
    spinner.fail(chalk.red('Health check failed'));
    throw error;
  }
}

/**
 * QW-2: Get timeout for health check mode
 */
function getTimeoutForMode(mode) {
  switch (mode) {
    case 'fast':
      return 2000; // 2s for liveness check
    case 'full':
      return 15000; // 15s for detailed diagnostics
    case 'on-error':
      return 500; // 500ms for immediate verification
    default:
      return 2000; // Default to fast mode
  }
}

/**
 * QW-2: Perform health check based on mode
 */
async function performHealthCheck(client, mode, timeoutMs) {
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), timeoutMs);

  try {
    switch (mode) {
      case 'fast':
        // Fast liveness check - just verify endpoint is responsive
        return await client.health({ signal: controller.signal, minimal: true });

      case 'full':
        // Full detailed health check with all components
        return await client.health({ signal: controller.signal, detailed: true });

      case 'on-error':
        // On-error verification - quick check of critical components only
        return await client.health({ signal: controller.signal, critical: true });

      default:
        return await client.health({ signal: controller.signal });
    }
  } finally {
    clearTimeout(timeout);
  }
}

async function watchHealth(globalOpts, options) {
  const interval = parseInt(options.interval) * 1000;

  console.log(chalk.blue(`Watching health status (${options.interval}s interval)...\n`));
  console.log(chalk.gray('Press Ctrl+C to stop\n'));

  const client = new RipTideClient({
    url: globalOpts.url,
    apiKey: globalOpts.apiKey
  });

  async function check() {
    try {
      const health = await client.health();

      // Clear console
      process.stdout.write('\x1Bc');

      console.log(chalk.blue.bold('🌊 RipTide Health Monitor\n'));
      console.log(formatHealth(health));
      console.log(chalk.gray(`\nLast updated: ${new Date().toLocaleTimeString()}`));
      console.log(chalk.gray(`Press Ctrl+C to stop`));

    } catch (error) {
      console.log(chalk.red(`✗ Health check failed: ${error.message}`));
    }
  }

  // Initial check
  await check();

  // Periodic checks
  setInterval(check, interval);
}
