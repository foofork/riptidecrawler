/**
 * Search command implementation
 */

import chalk from 'chalk';
import ora from 'ora';
import { writeFileSync } from 'fs';
import { RipTideClient } from '../utils/api-client.js';
import { formatSearchText, formatJSON } from '../utils/formatters.js';

export async function searchCommand(query, options, command) {
  const globalOpts = command.parent.opts();
  const spinner = ora(`Searching for "${query}"...`).start();

  try {
    const client = new RipTideClient({
      url: globalOpts.url,
      apiKey: globalOpts.apiKey
    });

    const result = await client.search(query, {
      limit: parseInt(options.limit),
      includeContent: options.includeContent
    });

    spinner.succeed(chalk.green(`Found ${result.results.length} result(s)`));

    // Format output
    let output;
    if (globalOpts.json || options.format === 'json') {
      output = formatJSON(result);
    } else if (options.format === 'markdown') {
      output = formatSearchMarkdown(result);
    } else {
      output = formatSearchText(result);
    }

    // Save to file if requested
    if (options.output) {
      writeFileSync(options.output, output);
      console.log(chalk.blue(`\n✓ Saved to ${options.output}`));
    } else {
      console.log(output);
    }

  } catch (error) {
    spinner.fail(chalk.red('Search failed'));
    throw error;
  }
}

function formatSearchMarkdown(result) {
  const output = [`# Search Results: ${result.query}\n`];

  result.results.forEach((item, index) => {
    output.push(`## ${index + 1}. ${item.title}\n`);
    output.push(`**URL**: ${item.url}\n`);

    if (item.snippet) {
      output.push(`${item.snippet}\n`);
    }

    output.push('---\n');
  });

  return output.join('\n');
}
