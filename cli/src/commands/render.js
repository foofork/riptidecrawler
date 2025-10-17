/**
 * Render command implementation
 * Renders URLs using headless browser
 */

import chalk from 'chalk';
import ora from 'ora';
import { RipTideClient } from '../utils/api-client.js';
import { formatJSON } from '../utils/formatters.js';

export async function renderCommand(url, options, command) {
  const globalOpts = command.parent.opts();
  const client = new RipTideClient({
    url: globalOpts.url,
    apiKey: globalOpts.apiKey
  });

  const spinner = ora('Rendering page...').start();

  try {
    const result = await client.render(url, {
      waitTime: options.waitTime || 2000,
      screenshot: options.screenshot || false
    });

    spinner.stop();

    if (globalOpts.json) {
      console.log(formatJSON(result));
    } else {
      console.log(chalk.blue.bold('\n🖼️  Page Rendered\n'));
      console.log(chalk.gray(`URL: ${chalk.white(url)}`));

      if (result.html) {
        console.log(chalk.gray(`HTML Length: ${chalk.white(result.html.length)} characters`));
      }

      if (result.screenshot) {
        console.log(chalk.green('✓ Screenshot captured'));
      }

      if (result.dom) {
        console.log(chalk.gray(`DOM Elements: ${chalk.white(result.dom.length || 'N/A')}`));
      }

      console.log();
    }

  } catch (error) {
    spinner.fail(chalk.red('Failed to render page'));
    throw error;
  }
}
