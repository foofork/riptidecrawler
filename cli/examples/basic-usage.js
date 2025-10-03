#!/usr/bin/env node

/**
 * Basic CLI usage examples
 * Run: node examples/basic-usage.js
 */

import { RipTideClient } from '../src/utils/api-client.js';
import chalk from 'chalk';

async function main() {
  console.log(chalk.blue.bold('\n🌊 RipTide CLI - Basic Usage Examples\n'));

  const client = new RipTideClient({
    url: 'http://localhost:8080'
  });

  try {
    // Example 1: Health Check
    console.log(chalk.yellow('Example 1: Health Check'));
    const health = await client.health();
    console.log(chalk.green(`✓ Status: ${health.status}`));
    console.log(chalk.gray(`  Version: ${health.version}\n`));

    // Example 2: Simple Crawl
    console.log(chalk.yellow('Example 2: Simple Crawl'));
    const crawlResult = await client.crawl(['https://example.com']);
    console.log(chalk.green(`✓ Crawled: ${crawlResult.results[0].url}`));
    console.log(chalk.gray(`  Title: ${crawlResult.results[0].document?.title}\n`));

    // Example 3: Search
    console.log(chalk.yellow('Example 3: Search'));
    const searchResult = await client.search('web scraping', { limit: 5 });
    console.log(chalk.green(`✓ Found ${searchResult.results.length} results`));
    searchResult.results.slice(0, 3).forEach((item, i) => {
      console.log(chalk.gray(`  ${i + 1}. ${item.title}`));
    });

    console.log(chalk.blue.bold('\n✓ All examples completed successfully!\n'));

  } catch (error) {
    console.error(chalk.red(`\n✗ Error: ${error.message}\n`));
    process.exit(1);
  }
}

main();
