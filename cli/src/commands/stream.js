/**
 * Stream command implementation
 */

import chalk from 'chalk';
import ora from 'ora';
import { createWriteStream } from 'fs';
import { RipTideClient } from '../utils/api-client.js';
import { formatJSON } from '../utils/formatters.js';

export async function streamCommand(urls, options, command) {
  const globalOpts = command.parent.opts();
  const spinner = ora('Starting stream...').start();

  try {
    const client = new RipTideClient({
      url: globalOpts.url,
      apiKey: globalOpts.apiKey
    });

    let writeStream;
    if (options.output) {
      writeStream = createWriteStream(options.output);
    }

    let count = 0;

    await client.streamCrawl(urls, {
      concurrency: parseInt(options.concurrency)
    }, (result) => {
      count++;
      spinner.text = `Received ${count}/${urls.length} results...`;

      const output = options.format === 'ndjson' || options.output
        ? JSON.stringify(result) + '\n'
        : formatStreamResult(result);

      if (writeStream) {
        writeStream.write(output);
      } else {
        spinner.stop();
        console.log(output);
        spinner.start();
      }
    });

    if (writeStream) {
      writeStream.end();
    }

    spinner.succeed(chalk.green(`Streamed ${count} result(s)`));

    if (options.output) {
      console.log(chalk.blue(`✓ Saved to ${options.output}`));
    }

  } catch (error) {
    spinner.fail(chalk.red('Stream failed'));
    throw error;
  }
}

function formatStreamResult(result) {
  const icon = result.error ? '✗' : '✓';
  const color = result.error ? chalk.red : chalk.green;

  let output = color(`${icon} ${result.url}`);

  if (result.document?.title) {
    output += chalk.gray(` - ${result.document.title}`);
  }

  if (result.error) {
    output += chalk.red(`\n  Error: ${result.error}`);
  }

  return output;
}
