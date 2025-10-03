/**
 * Batch command implementation
 */

import chalk from 'chalk';
import ora from 'ora';
import { readFileSync, writeFileSync } from 'fs';
import { RipTideClient } from '../utils/api-client.js';
import { formatJSON, formatNDJSON } from '../utils/formatters.js';

export async function batchCommand(file, options, command) {
  const globalOpts = command.parent.opts();
  const spinner = ora('Loading URLs...').start();

  try {
    // Read URLs from file
    const content = readFileSync(file, 'utf8');
    const urls = content
      .split('\n')
      .map(line => line.trim())
      .filter(line => line && !line.startsWith('#'));

    if (urls.length === 0) {
      throw new Error('No URLs found in file');
    }

    spinner.text = `Processing ${urls.length} URLs...`;

    const client = new RipTideClient({
      url: globalOpts.url,
      apiKey: globalOpts.apiKey
    });

    const results = [];
    const errors = [];
    let completed = 0;

    // Process in chunks based on concurrency
    const concurrency = parseInt(options.concurrency);
    const chunks = [];

    for (let i = 0; i < urls.length; i += concurrency) {
      chunks.push(urls.slice(i, i + concurrency));
    }

    for (const chunk of chunks) {
      try {
        const result = await client.crawl(chunk, {
          concurrency: chunk.length
        });

        results.push(...result.results);
        completed += chunk.length;

        spinner.text = `Processed ${completed}/${urls.length} URLs...`;

      } catch (error) {
        errors.push({
          urls: chunk,
          error: error.message
        });

        if (!options.continueOnError) {
          throw error;
        }
      }
    }

    spinner.succeed(chalk.green(`Processed ${urls.length} URLs`));

    // Format and output results
    let output;

    if (options.format === 'ndjson') {
      output = formatNDJSON(results);
    } else if (options.format === 'csv') {
      output = formatCSV(results);
    } else {
      output = formatJSON({ results, errors });
    }

    // Save to file if requested
    if (options.output) {
      writeFileSync(options.output, output);
      console.log(chalk.blue(`\n✓ Saved to ${options.output}`));
    } else {
      console.log(output);
    }

    // Show summary
    console.log(chalk.blue('\nBatch Summary:'));
    console.log(chalk.green(`  ✓ Successful: ${results.filter(r => !r.error).length}`));

    const failed = results.filter(r => r.error).length + errors.length;
    if (failed > 0) {
      console.log(chalk.red(`  ✗ Failed: ${failed}`));
    }

    const cached = results.filter(r => r.cached).length;
    if (cached > 0) {
      console.log(chalk.cyan(`  📦 Cached: ${cached}`));
    }

  } catch (error) {
    spinner.fail(chalk.red('Batch processing failed'));
    throw error;
  }
}

function formatCSV(results) {
  const headers = ['URL', 'Title', 'Status', 'Quality Score', 'Cached'];
  const rows = [headers.join(',')];

  results.forEach(result => {
    const row = [
      result.url,
      result.document?.title || '',
      result.error ? 'Error' : 'Success',
      result.quality_score ? Math.round(result.quality_score * 100) : '',
      result.cached ? 'Yes' : 'No'
    ];

    // Escape commas and quotes
    const escaped = row.map(cell => {
      const str = String(cell);
      if (str.includes(',') || str.includes('"')) {
        return `"${str.replace(/"/g, '""')}"`;
      }
      return str;
    });

    rows.push(escaped.join(','));
  });

  return rows.join('\n');
}
