/**
 * Spider command implementation
 */

import chalk from 'chalk';
import ora from 'ora';
import { writeFileSync } from 'fs';
import { RipTideClient } from '../utils/api-client.js';
import { formatJSON } from '../utils/formatters.js';

export async function spiderCommand(url, options, command) {
  const globalOpts = command.parent.opts();
  const spinner = ora('Starting spider...').start();

  try {
    const client = new RipTideClient({
      url: globalOpts.url,
      apiKey: globalOpts.apiKey
    });

    const result = await client.startSpider(url, {
      maxDepth: parseInt(options.maxDepth),
      maxPages: parseInt(options.maxPages)
    });

    spinner.succeed(chalk.green('Spider started'));

    // Display spider info
    console.log(chalk.blue('\nSpider Job:'));
    console.log(chalk.gray(`  Job ID: ${result.job_id || result.id}`));
    console.log(chalk.gray(`  Starting URL: ${url}`));
    console.log(chalk.gray(`  Max Depth: ${options.maxDepth}`));
    console.log(chalk.gray(`  Max Pages: ${options.maxPages}`));
    console.log(chalk.gray(`  Status: ${result.status}`));

    // Save job info if requested
    if (options.output) {
      writeFileSync(options.output, formatJSON(result));
      console.log(chalk.blue(`\n✓ Job info saved to ${options.output}`));
    }

    console.log(chalk.yellow('\nNote: Use "riptide worker jobs" to check spider progress'));

  } catch (error) {
    spinner.fail(chalk.red('Failed to start spider'));
    throw error;
  }
}
