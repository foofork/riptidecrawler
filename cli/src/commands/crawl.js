/**
 * Crawl command implementation
 */

import chalk from 'chalk';
import ora from 'ora';
import { writeFileSync } from 'fs';
import { RipTideClient } from '../utils/api-client.js';
import { formatCrawlText, formatCrawlMarkdown, formatJSON } from '../utils/formatters.js';

export async function crawlCommand(urls, options, command) {
  const globalOpts = command.parent.opts();
  const spinner = ora('Crawling URLs...').start();

  try {
    const client = new RipTideClient({
      url: globalOpts.url,
      apiKey: globalOpts.apiKey,
      timeout: parseInt(options.timeout) * 1000
    });

    const result = await client.crawl(urls, {
      concurrency: parseInt(options.concurrency),
      cacheMode: options.cache,
      extractMode: options.extract
    });

    spinner.succeed(chalk.green(`Crawled ${urls.length} URL(s)`));

    // Format output
    let output;
    if (globalOpts.json || options.format === 'json') {
      output = formatJSON(result);
    } else if (options.format === 'markdown') {
      output = formatCrawlMarkdown(result);
    } else {
      output = formatCrawlText(result);
    }

    // Save to file if requested
    if (options.output) {
      writeFileSync(options.output, output);
      console.log(chalk.blue(`\n✓ Saved to ${options.output}`));
    } else {
      console.log(output);
    }

    // Show summary
    if (!globalOpts.json && options.format !== 'json') {
      const successful = result.results.filter(r => !r.error).length;
      const failed = result.results.filter(r => r.error).length;
      const cached = result.results.filter(r => r.cached).length;

      console.log(chalk.blue('\nSummary:'));
      console.log(chalk.green(`  ✓ Successful: ${successful}`));
      if (failed > 0) {
        console.log(chalk.red(`  ✗ Failed: ${failed}`));
      }
      if (cached > 0) {
        console.log(chalk.cyan(`  📦 Cached: ${cached}`));
      }
    }

  } catch (error) {
    spinner.fail(chalk.red('Crawl failed'));
    throw error;
  }
}
