/**
 * Profiling command implementation
 */

import chalk from 'chalk';
import ora from 'ora';
import { writeFileSync } from 'fs';
import { RipTideClient } from '../utils/api-client.js';
import {
  formatMemoryProfile,
  formatCPUProfile,
  formatBottlenecks,
  formatAllocations,
  formatLeakDetection,
  formatSnapshot,
  formatJSON
} from '../utils/formatters.js';

export async function profilingCommand(subcommand, options, command) {
  const globalOpts = command.parent.opts();

  if (!subcommand) {
    console.log(chalk.red('✗ Profiling subcommand required'));
    console.log(chalk.gray('\nAvailable subcommands:'));
    console.log(chalk.cyan('  memory       ') + 'Show memory profile');
    console.log(chalk.cyan('  cpu          ') + 'Show CPU profile');
    console.log(chalk.cyan('  bottlenecks  ') + 'Identify performance bottlenecks');
    console.log(chalk.cyan('  allocations  ') + 'Show allocation metrics');
    console.log(chalk.cyan('  leaks        ') + 'Detect memory leaks');
    console.log(chalk.cyan('  snapshot     ') + 'Create heap snapshot');
    process.exit(1);
  }

  const client = new RipTideClient({
    url: globalOpts.url,
    apiKey: globalOpts.apiKey
  });

  try {
    let result;
    let spinner;
    let formatter;

    switch (subcommand) {
      case 'memory':
        spinner = ora('Fetching memory profile...').start();
        result = await client.getMemoryProfile();
        spinner.succeed(chalk.green('Memory profile retrieved'));
        formatter = formatMemoryProfile;
        break;

      case 'cpu':
        spinner = ora('Fetching CPU profile...').start();
        result = await client.getCPUProfile();
        spinner.succeed(chalk.green('CPU profile retrieved'));
        formatter = formatCPUProfile;
        break;

      case 'bottlenecks':
        spinner = ora('Analyzing performance bottlenecks...').start();
        result = await client.getBottlenecks();
        spinner.succeed(chalk.green('Bottleneck analysis complete'));
        formatter = formatBottlenecks;
        break;

      case 'allocations':
        spinner = ora('Fetching allocation metrics...').start();
        result = await client.getAllocations();
        spinner.succeed(chalk.green('Allocation metrics retrieved'));
        formatter = formatAllocations;
        break;

      case 'leaks':
        spinner = ora('Detecting memory leaks...').start();
        result = await client.detectLeaks();
        spinner.succeed(chalk.green('Leak detection complete'));
        formatter = formatLeakDetection;
        break;

      case 'snapshot':
        spinner = ora('Creating heap snapshot...').start();
        result = await client.createSnapshot();
        spinner.succeed(chalk.green('Heap snapshot created'));
        formatter = formatSnapshot;
        break;

      default:
        console.log(chalk.red(`✗ Unknown subcommand: ${subcommand}`));
        console.log(chalk.gray('Run "riptide profiling" to see available subcommands'));
        process.exit(1);
    }

    // Format output
    let output;
    if (globalOpts.json || options.format === 'json') {
      output = formatJSON(result);
    } else {
      output = formatter(result);
    }

    // Save to file if requested
    if (options.output) {
      writeFileSync(options.output, output);
      console.log(chalk.blue(`\n✓ Saved to ${options.output}`));
    } else {
      console.log('\n' + output);
    }

    // Show warnings if any
    if (!globalOpts.json && options.format !== 'json') {
      if (result.warnings && result.warnings.length > 0) {
        console.log(chalk.yellow('\n⚠ Warnings:'));
        result.warnings.forEach(warning => {
          console.log(chalk.yellow(`  • ${warning}`));
        });
      }

      // Show recommendations for specific subcommands
      if (subcommand === 'leaks' && result.potential_leaks && result.potential_leaks.length > 0) {
        console.log(chalk.blue('\n💡 Recommendations:'));
        console.log(chalk.gray('  • Run "riptide profiling snapshot" to capture detailed heap state'));
        console.log(chalk.gray('  • Consider running leak detection multiple times to confirm patterns'));
      }

      if (subcommand === 'bottlenecks' && result.hotspots && result.hotspots.length > 0) {
        console.log(chalk.blue('\n💡 Recommendations:'));
        console.log(chalk.gray('  • Focus on optimizing the top 3 hotspots first'));
        console.log(chalk.gray('  • Use "riptide profiling cpu" for detailed CPU analysis'));
      }

      if (subcommand === 'snapshot') {
        console.log(chalk.blue('\n💡 Next steps:'));
        console.log(chalk.gray(`  • Snapshot ID: ${result.snapshot_id}`));
        console.log(chalk.gray('  • Download and analyze with Chrome DevTools'));
        console.log(chalk.gray(`  • File size: ${(result.size_bytes / 1024 / 1024).toFixed(2)} MB`));
      }
    }

  } catch (error) {
    console.log(chalk.red(`✗ Profiling failed: ${error.message}`));
    process.exit(1);
  }
}
