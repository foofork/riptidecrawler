/**
 * Monitor command implementation
 */

import chalk from 'chalk';
import { RipTideClient } from '../utils/api-client.js';
import { formatMonitoring } from '../utils/formatters.js';

export async function monitorCommand(options, command) {
  const globalOpts = command.parent.opts();
  const interval = parseInt(options.interval) * 1000;

  console.log(chalk.blue.bold('🌊 RipTide Monitor\n'));
  console.log(chalk.gray(`Update interval: ${options.interval}s`));
  console.log(chalk.gray('Press Ctrl+C to stop\n'));

  const client = new RipTideClient({
    url: globalOpts.url,
    apiKey: globalOpts.apiKey
  });

  async function check() {
    try {
      const data = {};

      // Get health score
      if (options.score || !options.metrics) {
        data.healthScore = await client.healthScore();
      }

      // Get performance metrics
      if (options.metrics) {
        data.performance = await client.performanceReport();
      }

      // Clear console
      if (!globalOpts.json) {
        process.stdout.write('\x1Bc');
        console.log(chalk.blue.bold('🌊 RipTide Monitor\n'));
      }

      console.log(formatMonitoring(data));

      if (!globalOpts.json) {
        console.log(chalk.gray(`\nLast updated: ${new Date().toLocaleTimeString()}`));
        console.log(chalk.gray('Press Ctrl+C to stop'));
      }

    } catch (error) {
      console.log(chalk.red(`✗ Monitor check failed: ${error.message}`));
    }
  }

  // Initial check
  await check();

  // Periodic checks
  setInterval(check, interval);
}
