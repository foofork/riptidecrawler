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

// ============================================================================
// PROFILING FORMATTERS
// ============================================================================

/**
 * Format memory profile data
 */
export function formatMemoryProfile(data) {
  const table = new Table({
    head: [chalk.blue('Metric'), chalk.blue('Value'), chalk.blue('Status')],
    colWidths: [25, 20, 15]
  });

  const rssMB = data.rss ? (data.rss / 1024 / 1024).toFixed(2) : 'N/A';
  const heapMB = data.heapUsed ? (data.heapUsed / 1024 / 1024).toFixed(2) : 'N/A';
  const heapTotalMB = data.heapTotal ? (data.heapTotal / 1024 / 1024).toFixed(2) : 'N/A';
  const externalMB = data.external ? (data.external / 1024 / 1024).toFixed(2) : 'N/A';

  // Determine status based on thresholds
  const heapUsagePercent = data.heapUsed && data.heapTotal
    ? (data.heapUsed / data.heapTotal * 100).toFixed(1)
    : 0;

  let status;
  if (heapUsagePercent > 90) {
    status = chalk.red('Critical');
  } else if (heapUsagePercent > 75) {
    status = chalk.yellow('Warning');
  } else {
    status = chalk.green('Good');
  }

  table.push(
    ['RSS Memory', `${rssMB} MB`, status],
    ['Heap Used', `${heapMB} MB`, status],
    ['Heap Total', `${heapTotalMB} MB`, chalk.gray('—')],
    ['External', `${externalMB} MB`, chalk.gray('—')],
    ['Heap Usage', `${heapUsagePercent}%`, status]
  );

  return table.toString();
}

/**
 * Format CPU profile data
 */
export function formatCPUProfile(data) {
  const table = new Table({
    head: [chalk.blue('Metric'), chalk.blue('Value'), chalk.blue('Status')],
    colWidths: [25, 20, 15]
  });

  const cpuUsage = data.cpuUsage !== undefined ? `${data.cpuUsage.toFixed(2)}%` : 'N/A';
  const loadAvg = data.loadAverage || [0, 0, 0];
  const loadAvgStr = `${loadAvg[0].toFixed(2)}, ${loadAvg[1].toFixed(2)}, ${loadAvg[2].toFixed(2)}`;

  // Determine status
  let status;
  if (data.cpuUsage > 80) {
    status = chalk.red('Critical');
  } else if (data.cpuUsage > 60) {
    status = chalk.yellow('Warning');
  } else {
    status = chalk.green('Good');
  }

  table.push(
    ['CPU Usage', cpuUsage, status],
    ['Load Average (1m, 5m, 15m)', loadAvgStr, chalk.gray('—')],
    ['User Time', data.user ? `${data.user.toFixed(2)}ms` : 'N/A', chalk.gray('—')],
    ['System Time', data.system ? `${data.system.toFixed(2)}ms` : 'N/A', chalk.gray('—')]
  );

  return table.toString();
}

/**
 * Format bottleneck analysis
 */
export function formatBottlenecks(data) {
  const output = [chalk.blue.bold('\n🔍 Performance Bottlenecks\n')];

  if (!data.hotspots || data.hotspots.length === 0) {
    return chalk.gray('No bottlenecks detected');
  }

  data.hotspots.forEach((hotspot, index) => {
    const percentage = hotspot.percentage ? `${hotspot.percentage.toFixed(2)}%` : 'N/A';
    const color = hotspot.percentage > 20 ? chalk.red : hotspot.percentage > 10 ? chalk.yellow : chalk.gray;

    output.push(color(`${index + 1}. ${hotspot.function || 'Unknown'}`));
    output.push(color(`   Time: ${percentage} | Calls: ${hotspot.calls || 0}`));

    if (hotspot.file) {
      output.push(chalk.gray(`   Location: ${hotspot.file}:${hotspot.line || '?'}`));
    }
    output.push('');
  });

  return output.join('\n');
}

/**
 * Format memory allocation analysis
 */
export function formatAllocations(data) {
  const output = [chalk.blue.bold('\n📊 Memory Allocations\n')];

  // Efficiency score
  const efficiency = data.efficiency !== undefined ? data.efficiency : 0;
  const efficiencyColor = efficiency >= 80 ? chalk.green : efficiency >= 60 ? chalk.yellow : chalk.red;
  output.push(efficiencyColor(`Efficiency Score: ${efficiency.toFixed(1)}/100`));

  // Fragmentation
  const fragmentation = data.fragmentation !== undefined ? `${data.fragmentation.toFixed(2)}%` : 'N/A';
  const fragColor = data.fragmentation > 30 ? chalk.red : data.fragmentation > 15 ? chalk.yellow : chalk.green;
  output.push(fragColor(`Fragmentation: ${fragmentation}\n`));

  // Top allocators
  if (data.topAllocators && data.topAllocators.length > 0) {
    const table = new Table({
      head: [chalk.blue('Function'), chalk.blue('Allocated'), chalk.blue('Count')],
      colWidths: [30, 15, 10]
    });

    data.topAllocators.forEach(allocator => {
      const sizeMB = allocator.size ? (allocator.size / 1024 / 1024).toFixed(2) : 'N/A';
      table.push([
        allocator.function || 'Unknown',
        `${sizeMB} MB`,
        allocator.count || 0
      ]);
    });

    output.push(table.toString());
  }

  return output.join('\n');
}

/**
 * Format leak detection results
 */
export function formatLeakDetection(data) {
  const output = [chalk.blue.bold('\n🔍 Memory Leak Detection\n')];

  const leaks = data.potential_leaks || data.potentialLeaks || [];

  if (leaks.length === 0) {
    return chalk.green('✓ No memory leaks detected');
  }

  output.push(chalk.yellow(`Found ${leaks.length} potential leak(s):\n`));

  leaks.forEach((leak, index) => {
    const riskColor = leak.risk === 'high' ? chalk.red : leak.risk === 'medium' ? chalk.yellow : chalk.green;
    const sizeMB = leak.size ? (leak.size / 1024 / 1024).toFixed(2) : 'N/A';

    output.push(riskColor(`${index + 1}. ${leak.type || 'Unknown'} [${leak.risk || 'low'} risk]`));
    output.push(riskColor(`   Size: ${sizeMB} MB | Growth: ${leak.growth || 0}%`));

    if (leak.location) {
      output.push(chalk.gray(`   Location: ${leak.location}`));
    }
    output.push('');
  });

  return output.join('\n');
}

/**
 * Format profiling snapshot
 */
export function formatSnapshot(data) {
  const table = new Table({
    head: [chalk.blue('Property'), chalk.blue('Value')],
    colWidths: [25, 35]
  });

  const sizeMB = (data.size_bytes || data.size) ? ((data.size_bytes || data.size) / 1024 / 1024).toFixed(2) : 'N/A';
  const timestamp = data.timestamp ? new Date(data.timestamp).toLocaleString() : 'N/A';
  const status = data.status === 'complete' ? chalk.green('Complete') : chalk.yellow(data.status || 'Unknown');

  table.push(
    ['Snapshot ID', data.snapshot_id || data.id || 'N/A'],
    ['Size', `${sizeMB} MB`],
    ['Status', status],
    ['Timestamp', timestamp],
    ['Type', data.type || 'heap'],
    ['Nodes', data.node_count || data.nodeCount || 'N/A']
  );

  return table.toString();
}

// ============================================================================
// RESOURCES FORMATTERS
// ============================================================================

/**
 * Format resource status overview
 */
export function formatResourceStatus(data) {
  const table = new Table({
    head: [chalk.blue('Resource'), chalk.blue('Status'), chalk.blue('Usage'), chalk.blue('Capacity')],
    colWidths: [20, 12, 15, 15]
  });

  const resources = data.resources || {};

  Object.entries(resources).forEach(([name, info]) => {
    const status = info.healthy ? chalk.green('Healthy') : chalk.red('Degraded');
    const usage = info.current !== undefined ? info.current : 'N/A';
    const capacity = info.max !== undefined ? `${info.current}/${info.max}` : 'N/A';

    table.push([name, status, usage, capacity]);
  });

  if (Object.keys(resources).length === 0) {
    return chalk.gray('No resource data available');
  }

  return table.toString();
}

/**
 * Format browser pool status
 */
export function formatBrowserPool(data) {
  const table = new Table({
    head: [chalk.blue('Metric'), chalk.blue('Value'), chalk.blue('Status')],
    colWidths: [25, 20, 15]
  });

  const active = data.active || 0;
  const idle = data.idle || 0;
  const total = data.total || active + idle;
  const max = data.max || total;
  const capacityPercent = max > 0 ? ((total / max) * 100).toFixed(1) : 0;

  let status;
  if (capacityPercent > 90) {
    status = chalk.red('Critical');
  } else if (capacityPercent > 75) {
    status = chalk.yellow('Warning');
  } else {
    status = chalk.green('Good');
  }

  table.push(
    ['Active Browsers', active, status],
    ['Idle Browsers', idle, chalk.gray('—')],
    ['Total Browsers', total, status],
    ['Max Capacity', max, chalk.gray('—')],
    ['Capacity Usage', `${capacityPercent}%`, status]
  );

  return table.toString();
}

/**
 * Format rate limiter status
 */
export function formatRateLimiter(data) {
  const table = new Table({
    head: [chalk.blue('Metric'), chalk.blue('Value'), chalk.blue('Status')],
    colWidths: [25, 20, 15]
  });

  const current = data.currentRate || 0;
  const limit = data.limit || 100;
  const throttled = data.throttled || 0;
  const usagePercent = limit > 0 ? ((current / limit) * 100).toFixed(1) : 0;

  let status;
  if (usagePercent > 90 || throttled > 10) {
    status = chalk.red('Throttling');
  } else if (usagePercent > 75) {
    status = chalk.yellow('Warning');
  } else {
    status = chalk.green('Good');
  }

  table.push(
    ['Current Rate', `${current} req/s`, status],
    ['Rate Limit', `${limit} req/s`, chalk.gray('—')],
    ['Usage', `${usagePercent}%`, status],
    ['Throttled Requests', throttled, throttled > 0 ? chalk.yellow('Active') : chalk.gray('—')]
  );

  return table.toString();
}

/**
 * Format resource memory breakdown
 */
export function formatResourceMemory(data) {
  const table = new Table({
    head: [chalk.blue('Component'), chalk.blue('Memory (MB)'), chalk.blue('Percentage')],
    colWidths: [25, 15, 15]
  });

  const components = data.components || {};
  const total = data.total || 0;

  Object.entries(components).forEach(([name, memory]) => {
    const memoryMB = (memory / 1024 / 1024).toFixed(2);
    const percentage = total > 0 ? ((memory / total) * 100).toFixed(1) : 0;
    table.push([name, memoryMB, `${percentage}%`]);
  });

  if (Object.keys(components).length === 0) {
    return chalk.gray('No memory data available');
  }

  return table.toString();
}

/**
 * Format resource performance metrics
 */
export function formatResourcePerformance(data) {
  const table = new Table({
    head: [chalk.blue('Metric'), chalk.blue('Value'), chalk.blue('Status')],
    colWidths: [25, 20, 15]
  });

  const throughput = data.throughput || 0;
  const p50 = data.latencyP50 || 0;
  const p95 = data.latencyP95 || 0;
  const p99 = data.latencyP99 || 0;

  let status;
  if (p95 > 1000) {
    status = chalk.red('Slow');
  } else if (p95 > 500) {
    status = chalk.yellow('Moderate');
  } else {
    status = chalk.green('Fast');
  }

  table.push(
    ['Throughput', `${throughput.toFixed(2)} req/s`, chalk.gray('—')],
    ['Latency P50', `${p50.toFixed(2)} ms`, status],
    ['Latency P95', `${p95.toFixed(2)} ms`, status],
    ['Latency P99', `${p99.toFixed(2)} ms`, status]
  );

  return table.toString();
}

/**
 * Format PDF resources status
 */
export function formatPDFResources(data) {
  const table = new Table({
    head: [chalk.blue('Metric'), chalk.blue('Value'), chalk.blue('Status')],
    colWidths: [25, 20, 15]
  });

  const queueSize = data.queueSize || 0;
  const processing = data.processing || 0;
  const rate = data.processingRate || 0;
  const successRate = data.successRate !== undefined ? (data.successRate * 100).toFixed(1) : 0;

  let status;
  if (successRate < 80 || queueSize > 100) {
    status = chalk.red('Issues');
  } else if (successRate < 95 || queueSize > 50) {
    status = chalk.yellow('Warning');
  } else {
    status = chalk.green('Good');
  }

  table.push(
    ['Queue Size', queueSize, queueSize > 50 ? chalk.yellow('High') : chalk.gray('—')],
    ['Processing', processing, chalk.gray('—')],
    ['Processing Rate', `${rate.toFixed(2)} PDF/s`, chalk.gray('—')],
    ['Success Rate', `${successRate}%`, status]
  );

  return table.toString();
}

// ============================================================================
// LLM FORMATTERS
// ============================================================================

/**
 * Format LLM providers list
 */
export function formatProviders(data) {
  const table = new Table({
    head: [chalk.blue('Provider'), chalk.blue('Status'), chalk.blue('Models'), chalk.blue('Active')],
    colWidths: [20, 12, 30, 10]
  });

  const providers = data.providers || [];

  if (providers.length === 0) {
    return chalk.gray('No LLM providers configured');
  }

  providers.forEach(provider => {
    const status = provider.available ? chalk.green('Available') : chalk.red('Unavailable');
    const models = provider.models ? provider.models.join(', ').substring(0, 28) : 'N/A';
    const active = provider.active ? chalk.green('✓') : chalk.gray('—');

    table.push([
      provider.name || 'Unknown',
      status,
      models,
      active
    ]);
  });

  return table.toString();
}

/**
 * Format LLM configuration
 */
export function formatLLMConfig(config) {
  const table = new Table({
    head: [chalk.blue('Setting'), chalk.blue('Value')],
    colWidths: [30, 40]
  });

  // Safely handle different config structures
  const entries = config && typeof config === 'object' ? Object.entries(config) : [];

  if (entries.length === 0) {
    return chalk.gray('No LLM configuration available');
  }

  entries.forEach(([key, value]) => {
    // Format the value appropriately
    let displayValue = value;
    if (typeof value === 'object' && value !== null) {
      displayValue = JSON.stringify(value).substring(0, 38);
    } else if (typeof value === 'boolean') {
      displayValue = value ? chalk.green('enabled') : chalk.red('disabled');
    } else if (key.toLowerCase().includes('key') || key.toLowerCase().includes('token')) {
      // Mask sensitive values
      displayValue = '••••••••';
    }

    table.push([key, displayValue]);
  });

  return table.toString();
}

/**
 * Format LLM provider switch result
 */
export async function formatSwitchResult(result) {
  const boxen = (await import('boxen')).default;

  const message = result.success
    ? chalk.green(`✓ Successfully switched to ${result.provider}`)
    : chalk.red(`✗ Failed to switch provider: ${result.error || 'Unknown error'}`);

  const details = [];
  if (result.provider) {
    details.push(chalk.gray(`Provider: ${result.provider}`));
  }
  if (result.model) {
    details.push(chalk.gray(`Model: ${result.model}`));
  }
  if (result.previousProvider) {
    details.push(chalk.gray(`Previous: ${result.previousProvider}`));
  }

  const content = [message, '', ...details].join('\n');

  return boxen(content, {
    padding: 1,
    margin: 1,
    borderStyle: result.success ? 'round' : 'double',
    borderColor: result.success ? 'green' : 'red'
  });
}
