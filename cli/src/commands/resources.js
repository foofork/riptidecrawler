/**
 * Resources command implementation
 * Monitor and display resource status across different subsystems
 */

import chalk from 'chalk';
import Table from 'cli-table3';
import ora from 'ora';
import { writeFileSync } from 'fs';
import { RipTideClient } from '../utils/api-client.js';
import { formatJSON } from '../utils/formatters.js';

/**
 * Main resources command handler
 */
export async function resourcesCommand(options, command) {
  const globalOpts = command.parent.opts();
  const subcommand = command.args[0];

  const client = new RipTideClient({
    url: globalOpts.url,
    apiKey: globalOpts.apiKey
  });

  // Handle watch mode
  if (options.watch) {
    return watchResources(client, subcommand, options, globalOpts);
  }

  // Single execution
  await executeResourceCommand(client, subcommand, options, globalOpts);
}

/**
 * Execute resource command once
 */
async function executeResourceCommand(client, subcommand, options, globalOpts) {
  const spinner = ora('Fetching resource data...').start();

  try {
    let data;
    let formatted;

    switch (subcommand) {
      case 'status':
        data = await client.getResourceStatus();
        formatted = formatResourceStatus(data, options.format);
        break;

      case 'browser-pool':
        data = await client.getBrowserPoolStatus();
        formatted = formatBrowserPool(data, options.format);
        break;

      case 'rate-limiter':
        data = await client.getRateLimiterStatus();
        formatted = formatRateLimiter(data, options.format);
        break;

      case 'memory':
        data = await client.getResourceMemory();
        formatted = formatResourceMemory(data, options.format);
        break;

      case 'performance':
        data = await client.getResourcePerformance();
        formatted = formatResourcePerformance(data, options.format);
        break;

      case 'pdf':
        data = await client.getPDFResourceStatus();
        formatted = formatPDFResources(data, options.format);
        break;

      default:
        spinner.fail();
        console.log(chalk.red(`Unknown subcommand: ${subcommand}`));
        console.log(chalk.gray('Available: status, browser-pool, rate-limiter, memory, performance, pdf'));
        process.exit(1);
    }

    spinner.succeed('Resource data fetched');

    // Output
    console.log('\n' + formatted);

    // Save to file if requested
    if (options.output) {
      const outputData = options.format === 'json' ? formatJSON(data) : formatted;
      writeFileSync(options.output, outputData);
      console.log(chalk.gray(`\n💾 Saved to ${options.output}`));
    }

  } catch (error) {
    spinner.fail('Failed to fetch resource data');
    console.error(chalk.red(`Error: ${error.message}`));
    process.exit(1);
  }
}

/**
 * Watch mode for continuous monitoring
 */
async function watchResources(client, subcommand, options, globalOpts) {
  const interval = parseInt(options.interval) * 1000;

  console.log(chalk.blue.bold('🌊 RipTide Resource Monitor\n'));
  console.log(chalk.gray(`Subcommand: ${subcommand}`));
  console.log(chalk.gray(`Update interval: ${options.interval}s`));
  console.log(chalk.gray('Press Ctrl+C to stop\n'));

  async function check() {
    try {
      let data;
      let formatted;

      switch (subcommand) {
        case 'status':
          data = await client.getResourceStatus();
          formatted = formatResourceStatus(data, options.format);
          break;
        case 'browser-pool':
          data = await client.getBrowserPoolStatus();
          formatted = formatBrowserPool(data, options.format);
          break;
        case 'rate-limiter':
          data = await client.getRateLimiterStatus();
          formatted = formatRateLimiter(data, options.format);
          break;
        case 'memory':
          data = await client.getResourceMemory();
          formatted = formatResourceMemory(data, options.format);
          break;
        case 'performance':
          data = await client.getResourcePerformance();
          formatted = formatResourcePerformance(data, options.format);
          break;
        case 'pdf':
          data = await client.getPDFResourceStatus();
          formatted = formatPDFResources(data, options.format);
          break;
      }

      // Clear console and redraw
      if (options.format !== 'json') {
        process.stdout.write('\x1Bc');
        console.log(chalk.blue.bold('🌊 RipTide Resource Monitor\n'));
      }

      console.log(formatted);

      if (options.format !== 'json') {
        console.log(chalk.gray(`\nLast updated: ${new Date().toLocaleTimeString()}`));
        console.log(chalk.gray('Press Ctrl+C to stop'));
      }

    } catch (error) {
      console.log(chalk.red(`✗ Resource check failed: ${error.message}`));
    }
  }

  // Initial check
  await check();

  // Periodic checks
  setInterval(check, interval);
}

// ============================================================
// Formatters
// ============================================================

/**
 * Format overall resource status
 */
function formatResourceStatus(data, format) {
  if (format === 'json') return formatJSON(data);

  const table = new Table({
    head: [chalk.blue('Resource'), chalk.blue('Status'), chalk.blue('Details')],
    colWidths: [20, 15, 50]
  });

  Object.entries(data).forEach(([resource, info]) => {
    const status = getStatusIndicator(info.status || info.healthy);
    const details = formatResourceDetails(info);
    table.push([resource, status, details]);
  });

  return table.toString();
}

/**
 * Format browser pool metrics
 */
function formatBrowserPool(data, format) {
  if (format === 'json') return formatJSON(data);

  const table = new Table({
    head: [chalk.blue('Metric'), chalk.blue('Value')],
    colWidths: [30, 20]
  });

  const activeColor = data.active_browsers > data.max_browsers * 0.8 ? chalk.red : chalk.green;
  const utilizationPct = ((data.active_browsers / data.max_browsers) * 100).toFixed(1);

  table.push(
    ['Active Browsers', activeColor(data.active_browsers || 0)],
    ['Idle Browsers', chalk.gray(data.idle_browsers || 0)],
    ['Max Capacity', data.max_browsers || 0],
    ['Utilization', `${utilizationPct}%`],
    ['Total Spawned', data.total_spawned || 0],
    ['Total Recycled', data.total_recycled || 0],
    ['Queue Length', data.queue_length || 0],
    ['Avg Wait Time', `${data.avg_wait_time_ms || 0}ms`]
  );

  return table.toString();
}

/**
 * Format rate limiter status
 */
function formatRateLimiter(data, format) {
  if (format === 'json') return formatJSON(data);

  const table = new Table({
    head: [chalk.blue('Metric'), chalk.blue('Value')],
    colWidths: [30, 20]
  });

  const currentRate = data.current_rate || 0;
  const limit = data.rate_limit || 100;
  const utilizationPct = ((currentRate / limit) * 100).toFixed(1);
  const rateColor = utilizationPct > 90 ? chalk.red : utilizationPct > 70 ? chalk.yellow : chalk.green;

  table.push(
    ['Current Rate (req/s)', rateColor(currentRate.toFixed(2))],
    ['Rate Limit', limit],
    ['Utilization', `${utilizationPct}%`],
    ['Throttled Requests', data.throttled_count || 0],
    ['Allowed Requests', data.allowed_count || 0],
    ['Active Windows', data.active_windows || 0],
    ['Oldest Window', data.oldest_window_age ? `${data.oldest_window_age}s ago` : 'N/A']
  );

  return table.toString();
}

/**
 * Format resource memory usage
 */
function formatResourceMemory(data, format) {
  if (format === 'json') return formatJSON(data);

  const table = new Table({
    head: [chalk.blue('Component'), chalk.blue('Memory (MB)'), chalk.blue('% of Total')],
    colWidths: [25, 15, 15]
  });

  const total = data.total_bytes || 0;

  Object.entries(data.by_component || {}).forEach(([component, bytes]) => {
    const mb = (bytes / 1024 / 1024).toFixed(2);
    const pct = ((bytes / total) * 100).toFixed(1);
    const color = pct > 30 ? chalk.red : pct > 15 ? chalk.yellow : chalk.green;

    table.push([
      component,
      color(mb),
      `${pct}%`
    ]);
  });

  // Add total row
  table.push([
    chalk.bold('TOTAL'),
    chalk.bold((total / 1024 / 1024).toFixed(2)),
    chalk.bold('100%')
  ]);

  return table.toString();
}

/**
 * Format resource performance metrics
 */
function formatResourcePerformance(data, format) {
  if (format === 'json') return formatJSON(data);

  const table = new Table({
    head: [chalk.blue('Metric'), chalk.blue('Value')],
    colWidths: [30, 20]
  });

  table.push(
    ['Throughput (req/s)', chalk.green((data.throughput || 0).toFixed(2))],
    ['Avg Latency (ms)', chalk.cyan((data.avg_latency_ms || 0).toFixed(2))],
    ['P50 Latency (ms)', (data.p50_latency_ms || 0).toFixed(2)],
    ['P95 Latency (ms)', (data.p95_latency_ms || 0).toFixed(2)],
    ['P99 Latency (ms)', chalk.yellow((data.p99_latency_ms || 0).toFixed(2))],
    ['Success Rate', `${((data.success_rate || 0) * 100).toFixed(2)}%`],
    ['Error Rate', `${((data.error_rate || 0) * 100).toFixed(2)}%`],
    ['Total Requests', data.total_requests || 0],
    ['Active Requests', data.active_requests || 0]
  );

  return table.toString();
}

/**
 * Format PDF processing resources
 */
function formatPDFResources(data, format) {
  if (format === 'json') return formatJSON(data);

  const table = new Table({
    head: [chalk.blue('Metric'), chalk.blue('Value')],
    colWidths: [30, 20]
  });

  const queueColor = data.queue_length > 10 ? chalk.red : data.queue_length > 5 ? chalk.yellow : chalk.green;

  table.push(
    ['Active Workers', chalk.green(data.active_workers || 0)],
    ['Idle Workers', chalk.gray(data.idle_workers || 0)],
    ['Max Workers', data.max_workers || 0],
    ['Queue Length', queueColor(data.queue_length || 0)],
    ['Processing Rate', `${(data.processing_rate || 0).toFixed(2)} docs/s`],
    ['Total Processed', data.total_processed || 0],
    ['Failed Documents', data.failed_count || 0],
    ['Avg Processing Time', `${data.avg_processing_ms || 0}ms`]
  );

  return table.toString();
}

// ============================================================
// Helpers
// ============================================================

/**
 * Get status indicator with color
 */
function getStatusIndicator(status) {
  if (typeof status === 'boolean') {
    return status ? chalk.green('✓ Healthy') : chalk.red('✗ Unhealthy');
  }

  switch (status?.toLowerCase()) {
    case 'healthy':
    case 'ok':
    case 'good':
      return chalk.green('✓ Healthy');
    case 'degraded':
    case 'warning':
      return chalk.yellow('⚠ Warning');
    case 'unhealthy':
    case 'critical':
    case 'error':
      return chalk.red('✗ Critical');
    default:
      return chalk.gray('? Unknown');
  }
}

/**
 * Format resource details for display
 */
function formatResourceDetails(info) {
  const details = [];

  if (info.count !== undefined) details.push(`Count: ${info.count}`);
  if (info.capacity !== undefined) details.push(`Capacity: ${info.capacity}`);
  if (info.utilization !== undefined) details.push(`Util: ${(info.utilization * 100).toFixed(1)}%`);
  if (info.latency_ms !== undefined) details.push(`Latency: ${info.latency_ms}ms`);
  if (info.error_rate !== undefined) details.push(`Errors: ${(info.error_rate * 100).toFixed(1)}%`);

  return details.join(', ') || 'No details available';
}
