/**
 * Health command implementation
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

  const spinner = ora('Checking health...').start();

  try {
    const client = new RipTideClient({
      url: globalOpts.url,
      apiKey: globalOpts.apiKey
    });

    const health = await client.health();

    spinner.stop();

    if (globalOpts.json) {
      console.log(formatJSON(health));
    } else {
      console.log(formatHealth(health));
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
