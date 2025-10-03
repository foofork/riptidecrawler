/**
 * Output formatters
 * Format API responses for different output types
 */

import chalk from 'chalk';
import Table from 'cli-table3';
import stripAnsi from 'strip-ansi';

/**
 * Format crawl results as text
 */
export function formatCrawlText(result) {
  const output = [];

  result.results.forEach((item, index) => {
    output.push(chalk.blue(`\n[${ index + 1}/${result.results.length}] ${item.url}`));

    if (item.error) {
      output.push(chalk.red(`  ✗ Error: ${item.error}`));
    } else {
      output.push(chalk.green(`  ✓ Success`));

      if (item.document?.title) {
        output.push(chalk.gray(`  Title: ${item.document.title}`));
      }

      if (item.quality_score !== undefined) {
        const score = Math.round(item.quality_score * 100);
        const color = score >= 80 ? chalk.green : score >= 50 ? chalk.yellow : chalk.red;
        output.push(color(`  Quality: ${score}%`));
      }

      if (item.cached) {
        output.push(chalk.cyan(`  Source: Cached`));
      }
    }
  });

  return output.join('\n');
}

/**
 * Format crawl results as markdown
 */
export function formatCrawlMarkdown(result) {
  const output = [`# Crawl Results\n`];

  result.results.forEach((item, index) => {
    output.push(`## ${index + 1}. ${item.url}\n`);

    if (item.error) {
      output.push(`**Error**: ${item.error}\n`);
    } else {
      if (item.document?.title) {
        output.push(`**Title**: ${item.document.title}\n`);
      }

      if (item.document?.content) {
        output.push(`### Content\n`);
        output.push(item.document.content.substring(0, 500) + '...\n');
      }

      if (item.quality_score !== undefined) {
        output.push(`**Quality Score**: ${Math.round(item.quality_score * 100)}%\n`);
      }
    }

    output.push('---\n');
  });

  return stripAnsi(output.join('\n'));
}

/**
 * Format search results as text
 */
export function formatSearchText(result) {
  const output = [chalk.blue.bold(`\n🔍 Search Results: ${result.query}\n`)];

  result.results.forEach((item, index) => {
    output.push(chalk.yellow(`${index + 1}. ${item.title}`));
    output.push(chalk.gray(`   ${item.url}`));

    if (item.snippet) {
      output.push(chalk.white(`   ${item.snippet.substring(0, 100)}...`));
    }

    output.push('');
  });

  return output.join('\n');
}

/**
 * Format health status
 */
export function formatHealth(health) {
  const output = [];

  // Status
  const statusIcon = health.status === 'healthy' ? '✓' : '✗';
  const statusColor = health.status === 'healthy' ? chalk.green : chalk.red;
  output.push(statusColor(`${statusIcon} Status: ${health.status}`));

  // Version
  output.push(chalk.blue(`  Version: ${health.version}`));

  // Uptime
  const uptime = formatUptime(health.uptime);
  output.push(chalk.gray(`  Uptime: ${uptime}`));

  // Dependencies
  if (health.dependencies) {
    output.push(chalk.blue('\n  Dependencies:'));

    Object.entries(health.dependencies).forEach(([name, info]) => {
      const icon = info.status === 'healthy' ? '✓' : '✗';
      const color = info.status === 'healthy' ? chalk.green : chalk.red;
      output.push(color(`    ${icon} ${name}: ${info.status}`));

      if (info.latency_ms !== undefined) {
        output.push(chalk.gray(`       Latency: ${info.latency_ms}ms`));
      }
    });
  }

  // Metrics
  if (health.metrics) {
    output.push(chalk.blue('\n  Metrics:'));

    if (health.metrics.memory_usage_bytes) {
      const mb = (health.metrics.memory_usage_bytes / 1024 / 1024).toFixed(2);
      output.push(chalk.gray(`    Memory: ${mb} MB`));
    }

    if (health.metrics.requests_per_second !== undefined) {
      output.push(chalk.gray(`    RPS: ${health.metrics.requests_per_second.toFixed(2)}`));
    }

    if (health.metrics.total_requests !== undefined) {
      output.push(chalk.gray(`    Total Requests: ${health.metrics.total_requests}`));
    }
  }

  return output.join('\n');
}

/**
 * Format worker status
 */
export function formatWorkerStatus(status) {
  const table = new Table({
    head: [chalk.blue('Metric'), chalk.blue('Value')],
    colWidths: [25, 20]
  });

  table.push(
    ['Active Jobs', status.active_jobs || 0],
    ['Queued Jobs', status.queued_jobs || 0],
    ['Completed Jobs', status.completed_jobs || 0],
    ['Failed Jobs', status.failed_jobs || 0],
    ['Workers', status.worker_count || 0]
  );

  return table.toString();
}

/**
 * Format session list
 */
export function formatSessionList(sessions) {
  if (!sessions || sessions.length === 0) {
    return chalk.gray('No sessions found');
  }

  const table = new Table({
    head: [
      chalk.blue('ID'),
      chalk.blue('Name'),
      chalk.blue('Created'),
      chalk.blue('Status')
    ],
    colWidths: [25, 20, 20, 12]
  });

  sessions.forEach(session => {
    const age = formatAge(new Date(session.created_at));
    const status = session.active ? chalk.green('Active') : chalk.gray('Inactive');

    table.push([
      session.id.substring(0, 20),
      session.name,
      age,
      status
    ]);
  });

  return table.toString();
}

/**
 * Format monitoring data
 */
export function formatMonitoring(data) {
  const output = [];

  // Timestamp
  output.push(chalk.gray(`[${new Date().toLocaleTimeString()}]`));

  // Health score
  if (data.healthScore) {
    const score = data.healthScore.score;
    const color = score >= 80 ? chalk.green : score >= 50 ? chalk.yellow : chalk.red;
    output.push(color(`  Health Score: ${score}/100`));
  }

  // Performance
  if (data.performance) {
    output.push(chalk.blue('  Performance:'));

    if (data.performance.response_time_p50) {
      output.push(chalk.gray(`    P50: ${data.performance.response_time_p50}ms`));
    }

    if (data.performance.response_time_p95) {
      output.push(chalk.gray(`    P95: ${data.performance.response_time_p95}ms`));
    }

    if (data.performance.success_rate !== undefined) {
      output.push(chalk.gray(`    Success: ${(data.performance.success_rate * 100).toFixed(2)}%`));
    }
  }

  return output.join('\n');
}

/**
 * Helper: Format uptime
 */
function formatUptime(seconds) {
  const days = Math.floor(seconds / 86400);
  const hours = Math.floor((seconds % 86400) / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);

  if (days > 0) {
    return `${days}d ${hours}h ${minutes}m`;
  } else if (hours > 0) {
    return `${hours}h ${minutes}m`;
  } else {
    return `${minutes}m`;
  }
}

/**
 * Helper: Format age
 */
function formatAge(date) {
  const now = new Date();
  const diff = now - date;
  const minutes = Math.floor(diff / 60000);
  const hours = Math.floor(minutes / 60);
  const days = Math.floor(hours / 24);

  if (days > 0) return `${days}d ago`;
  if (hours > 0) return `${hours}h ago`;
  return `${minutes}m ago`;
}

/**
 * Format JSON output
 */
export function formatJSON(data) {
  return JSON.stringify(data, null, 2);
}

/**
 * Format NDJSON (newline-delimited JSON)
 */
export function formatNDJSON(items) {
  return items.map(item => JSON.stringify(item)).join('\n');
}
