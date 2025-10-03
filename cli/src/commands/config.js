/**
 * Config command implementation
 */

import chalk from 'chalk';
import Table from 'cli-table3';
import { getConfig, resetConfig } from '../utils/config.js';

export async function configCommand(subCommand, key, value) {
  const config = getConfig();
  const action = subCommand || 'list';

  switch (action) {
    case 'get':
      return getConfigValue(config, key);

    case 'set':
      return setConfigValue(config, key, value);

    case 'list':
      return listConfig(config);

    case 'reset':
      return resetConfiguration();

    default:
      console.log(chalk.red(`Unknown config command: ${action}`));
      process.exit(1);
  }
}

function getConfigValue(config, key) {
  if (!key) {
    console.log(chalk.red('Key is required'));
    process.exit(1);
  }

  const value = config.get(key);

  if (value === undefined) {
    console.log(chalk.yellow(`Config key '${key}' not found`));
  } else {
    console.log(value);
  }
}

function setConfigValue(config, key, value) {
  if (!key || value === undefined) {
    console.log(chalk.red('Key and value are required'));
    process.exit(1);
  }

  config.set(key, value);
  console.log(chalk.green(`✓ Set ${key} = ${value}`));
}

function listConfig(config) {
  const table = new Table({
    head: [chalk.blue('Key'), chalk.blue('Value')],
    colWidths: [30, 50]
  });

  const allConfig = config.store;

  Object.entries(allConfig).forEach(([key, value]) => {
    // Hide API keys
    const displayValue = key.includes('key') && value
      ? '***' + value.slice(-4)
      : String(value);

    table.push([key, displayValue]);
  });

  console.log(chalk.blue.bold('\n⚙️  Configuration\n'));
  console.log(table.toString());
  console.log(chalk.gray(`\nConfig file: ${config.path}`));
}

function resetConfiguration() {
  resetConfig();
  console.log(chalk.green('✓ Configuration reset to defaults'));
}
