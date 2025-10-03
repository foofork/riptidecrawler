/**
 * Session command implementation
 */

import chalk from 'chalk';
import ora from 'ora';
import { RipTideClient } from '../utils/api-client.js';
import { formatSessionList, formatJSON } from '../utils/formatters.js';

export async function sessionCommand(subCommand, options, command) {
  const globalOpts = command.parent.opts();
  const client = new RipTideClient({
    url: globalOpts.url,
    apiKey: globalOpts.apiKey
  });

  const action = subCommand || 'list';

  switch (action) {
    case 'list':
      return listSessions(client, globalOpts);

    case 'create':
      return createSession(client, globalOpts, options);

    case 'delete':
      return deleteSession(client, globalOpts, options);

    default:
      console.log(chalk.red(`Unknown session command: ${action}`));
      process.exit(1);
  }
}

async function listSessions(client, globalOpts) {
  const spinner = ora('Fetching sessions...').start();

  try {
    const sessions = await client.listSessions();
    spinner.stop();

    if (globalOpts.json) {
      console.log(formatJSON(sessions));
    } else {
      console.log(formatSessionList(sessions.sessions || []));
    }

  } catch (error) {
    spinner.fail(chalk.red('Failed to list sessions'));
    throw error;
  }
}

async function createSession(client, globalOpts, options) {
  const spinner = ora('Creating session...').start();

  try {
    const config = {};

    if (options.userAgent) {
      config.user_agent = options.userAgent;
    }

    if (options.cookie && options.cookie.length > 0) {
      config.cookies = options.cookie.map(c => {
        const [name, value] = c.split('=');
        return { name, value };
      });
    }

    const session = await client.createSession(options.name || 'cli-session', config);

    spinner.succeed(chalk.green('Session created'));

    if (globalOpts.json) {
      console.log(formatJSON(session));
    } else {
      console.log(chalk.blue('\nSession Details:'));
      console.log(chalk.gray(`  ID: ${session.id}`));
      console.log(chalk.gray(`  Name: ${session.name}`));
      console.log(chalk.gray(`  Created: ${new Date(session.created_at).toLocaleString()}`));
    }

  } catch (error) {
    spinner.fail(chalk.red('Failed to create session'));
    throw error;
  }
}

async function deleteSession(client, globalOpts, options) {
  const spinner = ora('Deleting session...').start();

  try {
    await client.deleteSession(options.id);
    spinner.succeed(chalk.green('Session deleted'));

  } catch (error) {
    spinner.fail(chalk.red('Failed to delete session'));
    throw error;
  }
}
