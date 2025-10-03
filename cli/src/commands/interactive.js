/**
 * Interactive mode implementation
 */

import chalk from 'chalk';
import inquirer from 'inquirer';
import { RipTideClient } from '../utils/api-client.js';
import { formatCrawlText, formatSearchText, formatHealth } from '../utils/formatters.js';

export async function interactiveCommand(options, command) {
  const globalOpts = command.parent.opts();

  console.log(chalk.blue.bold('\n🌊 RipTide Interactive Mode\n'));
  console.log(chalk.gray('Type "help" for available commands, "exit" to quit\n'));

  const client = new RipTideClient({
    url: globalOpts.url,
    apiKey: globalOpts.apiKey
  });

  let running = true;

  while (running) {
    const { action } = await inquirer.prompt([
      {
        type: 'list',
        name: 'action',
        message: 'What would you like to do?',
        choices: [
          { name: '🌐 Crawl URLs', value: 'crawl' },
          { name: '🔍 Search', value: 'search' },
          { name: '❤️  Check Health', value: 'health' },
          { name: '📊 Worker Status', value: 'worker' },
          { name: '📝 List Sessions', value: 'sessions' },
          { name: '🕷️  Start Spider', value: 'spider' },
          new inquirer.Separator(),
          { name: '❌ Exit', value: 'exit' },
        ]
      }
    ]);

    try {
      switch (action) {
        case 'crawl':
          await interactiveCrawl(client);
          break;

        case 'search':
          await interactiveSearch(client);
          break;

        case 'health':
          await interactiveHealth(client);
          break;

        case 'worker':
          await interactiveWorker(client);
          break;

        case 'sessions':
          await interactiveSessions(client);
          break;

        case 'spider':
          await interactiveSpider(client);
          break;

        case 'exit':
          running = false;
          console.log(chalk.blue('\n👋 Goodbye!\n'));
          break;

        default:
          console.log(chalk.red('Unknown command'));
      }

      if (running && action !== 'exit') {
        console.log('\n');
        await inquirer.prompt([
          {
            type: 'input',
            name: 'continue',
            message: 'Press Enter to continue...'
          }
        ]);
      }

    } catch (error) {
      console.log(chalk.red(`\n✗ Error: ${error.message}\n`));
    }
  }
}

async function interactiveCrawl(client) {
  const answers = await inquirer.prompt([
    {
      type: 'input',
      name: 'urls',
      message: 'Enter URLs (comma-separated):',
      validate: (input) => input.trim().length > 0 || 'At least one URL is required'
    },
    {
      type: 'number',
      name: 'concurrency',
      message: 'Concurrency level:',
      default: 3
    },
    {
      type: 'list',
      name: 'cacheMode',
      message: 'Cache mode:',
      choices: ['auto', 'read_write', 'read_only', 'write_only', 'disabled'],
      default: 'auto'
    }
  ]);

  const urls = answers.urls.split(',').map(u => u.trim());

  console.log(chalk.blue('\nCrawling...'));

  const result = await client.crawl(urls, {
    concurrency: answers.concurrency,
    cacheMode: answers.cacheMode
  });

  console.log(formatCrawlText(result));
}

async function interactiveSearch(client) {
  const answers = await inquirer.prompt([
    {
      type: 'input',
      name: 'query',
      message: 'Search query:',
      validate: (input) => input.trim().length > 0 || 'Query is required'
    },
    {
      type: 'number',
      name: 'limit',
      message: 'Number of results:',
      default: 10
    },
    {
      type: 'confirm',
      name: 'includeContent',
      message: 'Include full content?',
      default: false
    }
  ]);

  console.log(chalk.blue('\nSearching...'));

  const result = await client.search(answers.query, {
    limit: answers.limit,
    includeContent: answers.includeContent
  });

  console.log(formatSearchText(result));
}

async function interactiveHealth(client) {
  console.log(chalk.blue('\nChecking health...'));

  const health = await client.health();
  console.log(formatHealth(health));
}

async function interactiveWorker(client) {
  console.log(chalk.blue('\nFetching worker status...'));

  const status = await client.workerStatus();

  console.log(chalk.blue('\nWorker Status:'));
  console.log(chalk.gray(`  Active Jobs: ${status.active_jobs || 0}`));
  console.log(chalk.gray(`  Queued Jobs: ${status.queued_jobs || 0}`));
  console.log(chalk.gray(`  Completed: ${status.completed_jobs || 0}`));
  console.log(chalk.gray(`  Failed: ${status.failed_jobs || 0}`));
}

async function interactiveSessions(client) {
  console.log(chalk.blue('\nFetching sessions...'));

  const sessions = await client.listSessions();
  const sessionList = sessions.sessions || [];

  if (sessionList.length === 0) {
    console.log(chalk.gray('No sessions found'));
    return;
  }

  sessionList.forEach((session, index) => {
    console.log(chalk.yellow(`\n${index + 1}. ${session.name}`));
    console.log(chalk.gray(`   ID: ${session.id}`));
    console.log(chalk.gray(`   Status: ${session.active ? 'Active' : 'Inactive'}`));
  });
}

async function interactiveSpider(client) {
  const answers = await inquirer.prompt([
    {
      type: 'input',
      name: 'url',
      message: 'Starting URL:',
      validate: (input) => input.trim().length > 0 || 'URL is required'
    },
    {
      type: 'number',
      name: 'maxDepth',
      message: 'Maximum crawl depth:',
      default: 2
    },
    {
      type: 'number',
      name: 'maxPages',
      message: 'Maximum pages:',
      default: 10
    }
  ]);

  console.log(chalk.blue('\nStarting spider...'));

  const result = await client.startSpider(answers.url, {
    maxDepth: answers.maxDepth,
    maxPages: answers.maxPages
  });

  console.log(chalk.green('\n✓ Spider started'));
  console.log(chalk.gray(`  Job ID: ${result.job_id || result.id}`));
  console.log(chalk.gray(`  Status: ${result.status}`));
}
