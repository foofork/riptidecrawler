#!/usr/bin/env node

/**
 * RipTide CLI - Command-line interface for RipTide API
 *
 * Usage:
 *   riptide crawl <url>
 *   riptide search <query>
 *   riptide health
 *   riptide --help
 */

import { program } from 'commander';
import chalk from 'chalk';
import updateNotifier from 'update-notifier';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';
import { readFileSync } from 'fs';

// Get package.json for version
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const packageJson = JSON.parse(
  readFileSync(join(__dirname, '../package.json'), 'utf8')
);

// Check for updates
updateNotifier({ pkg: packageJson }).notify();

// Import commands
import { crawlCommand } from '../src/commands/crawl.js';
import { searchCommand } from '../src/commands/search.js';
import { healthCommand } from '../src/commands/health.js';
import { sessionCommand } from '../src/commands/session.js';
import { workerCommand } from '../src/commands/worker.js';
import { monitorCommand } from '../src/commands/monitor.js';
import { configCommand } from '../src/commands/config.js';
import { streamCommand } from '../src/commands/stream.js';
import { batchCommand } from '../src/commands/batch.js';
import { interactiveCommand } from '../src/commands/interactive.js';
import { spiderCommand } from '../src/commands/spider.js';
import { profilingCommand } from '../src/commands/profiling.js';
import { resourcesCommand } from '../src/commands/resources.js';
import { llmCommand } from '../src/commands/llm.js';

// Configure CLI
program
  .name('riptide')
  .description('🌊 RipTide CLI - Enterprise web crawler from the command line')
  .version(packageJson.version, '-v, --version', 'Output the current version')
  .option('-u, --url <url>', 'RipTide API URL', process.env.RIPTIDE_API_URL)
  .option('-k, --api-key <key>', 'API key for authentication', process.env.RIPTIDE_API_KEY)
  .option('--no-color', 'Disable colored output')
  .option('--json', 'Output raw JSON')
  .option('--debug', 'Enable debug mode')
  .hook('preAction', (thisCommand, actionCommand) => {
    // Global error handling
    process.on('unhandledRejection', (error) => {
      console.error(chalk.red('✗ Error:'), error.message);
      if (actionCommand.opts().debug) {
        console.error(error.stack);
      }
      process.exit(1);
    });
  });

// ============================================================================
// CORE COMMANDS
// ============================================================================

// Crawl command
program
  .command('crawl')
  .description('Crawl one or more URLs')
  .argument('<urls...>', 'URLs to crawl')
  .option('-c, --concurrency <number>', 'Concurrency level', '3')
  .option('--cache <mode>', 'Cache mode (auto|read_write|read_only|write_only|disabled)', 'auto')
  .option('-o, --output <file>', 'Output file (JSON)')
  .option('-f, --format <type>', 'Output format (json|markdown|text)', 'text')
  .option('--extract <mode>', 'Extraction mode (article|full)', 'article')
  .option('--timeout <seconds>', 'Request timeout', '30')
  .action(crawlCommand);

// Search command
program
  .command('search')
  .alias('s')
  .description('Deep search with content extraction')
  .argument('<query>', 'Search query')
  .option('-l, --limit <number>', 'Maximum results', '10')
  .option('--include-content', 'Include full content in results')
  .option('-o, --output <file>', 'Output file (JSON)')
  .option('-f, --format <type>', 'Output format (json|markdown|text)', 'text')
  .action(searchCommand);

// Health command
program
  .command('health')
  .description('Check API health status')
  .option('-w, --watch', 'Watch health status continuously')
  .option('-i, --interval <seconds>', 'Watch interval in seconds', '5')
  .action(healthCommand);

// ============================================================================
// STREAMING COMMANDS
// ============================================================================

// Stream command
program
  .command('stream')
  .description('Stream crawl results in real-time')
  .argument('<urls...>', 'URLs to crawl')
  .option('-c, --concurrency <number>', 'Concurrency level', '3')
  .option('-f, --format <type>', 'Output format (ndjson|text)', 'text')
  .option('-o, --output <file>', 'Output file (NDJSON)')
  .action(streamCommand);

// ============================================================================
// SESSION COMMANDS
// ============================================================================

// Session command
program
  .command('session')
  .description('Manage crawling sessions')
  .addCommand(
    program.createCommand('list')
      .description('List all sessions')
  )
  .addCommand(
    program.createCommand('create')
      .description('Create a new session')
      .argument('<name>', 'Session name')
      .option('--user-agent <ua>', 'Custom user agent')
      .option('--cookie <cookie>', 'Add cookie (can be used multiple times)', [])
  )
  .addCommand(
    program.createCommand('delete')
      .description('Delete a session')
      .argument('<id>', 'Session ID')
  )
  .action(sessionCommand);

// ============================================================================
// WORKER COMMANDS
// ============================================================================

// Worker command
program
  .command('worker')
  .alias('w')
  .description('Manage worker queue')
  .addCommand(
    program.createCommand('status')
      .description('Get worker status')
  )
  .addCommand(
    program.createCommand('jobs')
      .description('List active jobs')
  )
  .action(workerCommand);

// ============================================================================
// MONITORING COMMANDS
// ============================================================================

// Monitor command
program
  .command('monitor')
  .alias('m')
  .description('Monitor API health and performance')
  .option('-i, --interval <seconds>', 'Update interval', '30')
  .option('--score', 'Show health score')
  .option('--metrics', 'Show performance metrics')
  .action(monitorCommand);

// Profiling command
program
  .command('profiling')
  .alias('profile')
  .description('System profiling and diagnostics')
  .argument('[subcommand]', 'Subcommand (memory|cpu|bottlenecks|allocations|leaks|snapshot)')
  .option('-f, --format <type>', 'Output format (json|text)', 'text')
  .option('-o, --output <file>', 'Output file')
  .action(profilingCommand);

// ============================================================================
// RESOURCE MANAGEMENT COMMANDS
// ============================================================================

// Resources command
program
  .command('resources')
  .alias('res')
  .description('Monitor and manage system resources')
  .argument('[subcommand]', 'Subcommand (status|browser-pool|rate-limiter|memory|performance|pdf)')
  .option('-f, --format <type>', 'Output format (json|text|table)', 'table')
  .option('-o, --output <file>', 'Output file')
  .option('-w, --watch', 'Continuous monitoring mode')
  .option('-i, --interval <seconds>', 'Watch interval in seconds', '10')
  .action(resourcesCommand);

// ============================================================================
// LLM PROVIDER COMMANDS
// ============================================================================

// LLM command
program
  .command('llm')
  .description('Manage LLM provider configuration')
  .argument('[subcommand]', 'Subcommand (providers|switch|config)')
  .argument('[args...]', 'Additional arguments')
  .option('-f, --format <type>', 'Output format (json|text|table)', 'table')
  .action(llmCommand);

// ============================================================================
// SPIDER COMMANDS
// ============================================================================

// Spider command
program
  .command('spider')
  .description('Deep crawl starting from a URL')
  .argument('<url>', 'Starting URL')
  .option('-d, --max-depth <number>', 'Maximum crawl depth', '2')
  .option('-p, --max-pages <number>', 'Maximum pages to crawl', '10')
  .option('-o, --output <file>', 'Output file (JSON)')
  .action(spiderCommand);

// ============================================================================
// BATCH COMMANDS
// ============================================================================

// Batch command
program
  .command('batch')
  .alias('b')
  .description('Process URLs from a file')
  .argument('<file>', 'File containing URLs (one per line)')
  .option('-c, --concurrency <number>', 'Concurrency level', '5')
  .option('-o, --output <file>', 'Output file (JSON)')
  .option('-f, --format <type>', 'Output format (json|ndjson|csv)', 'json')
  .option('--continue-on-error', 'Continue processing on errors')
  .action(batchCommand);

// ============================================================================
// CONFIGURATION COMMANDS
// ============================================================================

// Config command
program
  .command('config')
  .description('Manage CLI configuration')
  .addCommand(
    program.createCommand('get')
      .description('Get configuration value')
      .argument('[key]', 'Configuration key')
  )
  .addCommand(
    program.createCommand('set')
      .description('Set configuration value')
      .argument('<key>', 'Configuration key')
      .argument('<value>', 'Configuration value')
  )
  .addCommand(
    program.createCommand('list')
      .description('List all configuration')
  )
  .addCommand(
    program.createCommand('reset')
      .description('Reset configuration to defaults')
  )
  .action(configCommand);

// ============================================================================
// INTERACTIVE MODE
// ============================================================================

// Interactive command
program
  .command('interactive')
  .alias('i')
  .description('Start interactive mode')
  .action(interactiveCommand);

// ============================================================================
// UTILITY COMMANDS
// ============================================================================

// Examples command
program
  .command('examples')
  .description('Show usage examples')
  .action(() => {
    console.log(chalk.blue.bold('\n🌊 RipTide CLI Examples\n'));

    const examples = [
      {
        title: 'Basic Crawling',
        commands: [
          'riptide crawl https://example.com',
          'riptide crawl https://example.com --format markdown',
          'riptide crawl https://example.com --output result.json',
        ]
      },
      {
        title: 'Multiple URLs',
        commands: [
          'riptide crawl https://example.com https://example.org',
          'riptide crawl https://example.com https://example.org --concurrency 5',
        ]
      },
      {
        title: 'Search',
        commands: [
          'riptide search "web scraping tutorials"',
          'riptide search "python" --limit 20 --include-content',
        ]
      },
      {
        title: 'Streaming',
        commands: [
          'riptide stream https://example.com https://example.org',
          'riptide stream https://example.com --output results.ndjson',
        ]
      },
      {
        title: 'Batch Processing',
        commands: [
          'riptide batch urls.txt',
          'riptide batch urls.txt --concurrency 10 --output results.json',
        ]
      },
      {
        title: 'Monitoring',
        commands: [
          'riptide health',
          'riptide health --watch',
          'riptide monitor --interval 30',
        ]
      },
      {
        title: 'Configuration',
        commands: [
          'riptide config set api-url http://localhost:8080',
          'riptide config set api-key YOUR_API_KEY',
          'riptide config list',
        ]
      },
      {
        title: 'Interactive Mode',
        commands: [
          'riptide interactive',
        ]
      },
    ];

    examples.forEach(({ title, commands }) => {
      console.log(chalk.yellow(`\n${title}:`));
      commands.forEach(cmd => {
        console.log(chalk.gray(`  $ ${cmd}`));
      });
    });

    console.log('\n');
  });

// Parse arguments
program.parse();
