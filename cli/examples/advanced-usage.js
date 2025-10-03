#!/usr/bin/env node

/**
 * Advanced CLI usage examples
 * Run: node examples/advanced-usage.js
 */

import { RipTideClient } from '../src/utils/api-client.js';
import chalk from 'chalk';

async function main() {
  console.log(chalk.blue.bold('\n🌊 RipTide CLI - Advanced Usage Examples\n'));

  const client = new RipTideClient({
    url: 'http://localhost:8080'
  });

  try {
    // Example 1: Session Management
    console.log(chalk.yellow('Example 1: Session Management'));

    const session = await client.createSession('example-session', {
      user_agent: 'RipTide-CLI-Example/1.0'
    });

    console.log(chalk.green(`✓ Created session: ${session.id}`));

    // Use session for crawling
    const crawlResult = await client.crawl(['https://example.com'], {
      sessionId: session.id
    });

    console.log(chalk.gray(`  Crawled with session: ${crawlResult.results[0].url}\n`));

    // Example 2: Streaming
    console.log(chalk.yellow('Example 2: Streaming Crawl'));

    const urls = [
      'https://example.com',
      'https://example.org'
    ];

    let streamCount = 0;

    await client.streamCrawl(urls, {
      concurrency: 2
    }, (result) => {
      streamCount++;
      console.log(chalk.green(`  ✓ [${streamCount}/${urls.length}] ${result.url}`));
    });

    console.log(chalk.gray(`  Streamed ${streamCount} results\n`));

    // Example 3: Worker Status
    console.log(chalk.yellow('Example 3: Worker Status'));

    const workerStatus = await client.workerStatus();
    console.log(chalk.green(`✓ Worker status retrieved`));
    console.log(chalk.gray(`  Active jobs: ${workerStatus.active_jobs || 0}`));
    console.log(chalk.gray(`  Queued jobs: ${workerStatus.queued_jobs || 0}\n`));

    // Example 4: Health Score
    console.log(chalk.yellow('Example 4: Health Score'));

    const healthScore = await client.healthScore();
    console.log(chalk.green(`✓ Health score: ${healthScore.score}/100`));

    // Cleanup
    await client.deleteSession(session.id);
    console.log(chalk.gray(`\n✓ Cleaned up session\n`));

    console.log(chalk.blue.bold('✓ All advanced examples completed!\n'));

  } catch (error) {
    console.error(chalk.red(`\n✗ Error: ${error.message}\n`));
    process.exit(1);
  }
}

main();
