/**
 * Worker command implementation
 */

import chalk from 'chalk';
import ora from 'ora';
import { RipTideClient } from '../utils/api-client.js';
import { formatWorkerStatus, formatJSON } from '../utils/formatters.js';

export async function workerCommand(subCommand, options, command) {
  const globalOpts = command.parent.opts();
  const client = new RipTideClient({
    url: globalOpts.url,
    apiKey: globalOpts.apiKey
  });

  const action = subCommand || 'status';

  switch (action) {
    case 'status':
      return getWorkerStatus(client, globalOpts);

    case 'jobs':
      return listJobs(client, globalOpts);

    default:
      console.log(chalk.red(`Unknown worker command: ${action}`));
      process.exit(1);
  }
}

async function getWorkerStatus(client, globalOpts) {
  const spinner = ora('Fetching worker status...').start();

  try {
    const status = await client.workerStatus();
    spinner.stop();

    if (globalOpts.json) {
      console.log(formatJSON(status));
    } else {
      console.log(chalk.blue.bold('\n🔧 Worker Status\n'));
      console.log(formatWorkerStatus(status));
    }

  } catch (error) {
    spinner.fail(chalk.red('Failed to get worker status'));
    throw error;
  }
}

async function listJobs(client, globalOpts) {
  const spinner = ora('Fetching jobs...').start();

  try {
    // This would need a new API endpoint
    const jobs = await client.client.get('/workers/jobs');
    spinner.stop();

    if (globalOpts.json) {
      console.log(formatJSON(jobs));
    } else {
      console.log(chalk.blue.bold('\n📋 Active Jobs\n'));
      // Format jobs table
      console.log(formatJSON(jobs));
    }

  } catch (error) {
    spinner.fail(chalk.red('Failed to list jobs'));
    throw error;
  }
}
