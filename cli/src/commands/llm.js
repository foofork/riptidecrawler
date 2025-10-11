/**
 * LLM command implementation
 * Manages LLM provider configuration and switching
 */

import chalk from 'chalk';
import ora from 'ora';
import inquirer from 'inquirer';
import boxen from 'boxen';
import { RipTideClient } from '../utils/api-client.js';
import { formatProviders, formatLLMConfig, formatJSON } from '../utils/formatters.js';

// Allowed config keys for validation
const ALLOWED_CONFIG_KEYS = [
  'model',
  'temperature',
  'max_tokens',
  'top_p',
  'frequency_penalty',
  'presence_penalty',
  'timeout',
  'retry_attempts'
];

export async function llmCommand(options, command) {
  const globalOpts = command.parent.opts();
  const client = new RipTideClient({
    url: globalOpts.url,
    apiKey: globalOpts.apiKey
  });

  const subCommand = command.args[0];
  const format = options.format || 'table';

  switch (subCommand) {
    case 'providers':
      return listProviders(client, globalOpts, format);

    case 'switch':
      return switchProvider(client, globalOpts, command.args[1], format);

    case 'config':
      return handleConfigCommand(client, globalOpts, command.args.slice(1), format);

    default:
      console.log(chalk.red(`Unknown llm command: ${subCommand || '(none)'}`));
      console.log(chalk.gray('\nAvailable commands:'));
      console.log(chalk.gray('  providers           List available LLM providers'));
      console.log(chalk.gray('  switch <provider>   Switch active LLM provider'));
      console.log(chalk.gray('  config get [key]    Get LLM configuration'));
      console.log(chalk.gray('  config set <key> <value>  Set LLM configuration'));
      process.exit(1);
  }
}

/**
 * List available LLM providers
 */
async function listProviders(client, globalOpts, format) {
  const spinner = ora('Fetching LLM providers...').start();

  try {
    const data = await client.listLLMProviders();
    spinner.stop();

    if (format === 'json' || globalOpts.json) {
      console.log(formatJSON(data));
    } else {
      console.log(chalk.blue.bold('\n🤖 LLM Providers\n'));
      console.log(formatProviders(data));

      // Highlight current provider if available
      const activeProvider = data.providers?.find(p => p.active);
      if (activeProvider) {
        console.log(boxen(
          chalk.green(`✓ Active Provider: ${activeProvider.name}`),
          { padding: 0.5, margin: { top: 1 }, borderColor: 'green', borderStyle: 'round' }
        ));
      }
    }
  } catch (error) {
    spinner.fail(chalk.red('Failed to fetch providers'));
    throw error;
  }
}

/**
 * Switch active LLM provider
 */
async function switchProvider(client, globalOpts, provider, format) {
  // If no provider specified, show interactive selection
  if (!provider) {
    return await interactiveSwitchProvider(client, globalOpts, format);
  }

  // Validate provider name
  if (typeof provider !== 'string' || provider.trim().length === 0) {
    console.log(chalk.red('Error: Invalid provider name'));
    process.exit(1);
  }

  // Confirm switch
  const { confirm } = await inquirer.prompt([
    {
      type: 'confirm',
      name: 'confirm',
      message: `Switch to provider "${provider}"?`,
      default: true
    }
  ]);

  if (!confirm) {
    console.log(chalk.yellow('Switch cancelled'));
    return;
  }

  const spinner = ora(`Switching to ${provider}...`).start();

  try {
    const result = await client.switchLLMProvider(provider);
    spinner.stop();

    if (format === 'json' || globalOpts.json) {
      console.log(formatJSON(result));
    } else {
      console.log(formatSwitchResult(result));
    }
  } catch (error) {
    spinner.fail(chalk.red(`Failed to switch to ${provider}`));
    throw error;
  }
}

/**
 * Interactive provider selection
 */
async function interactiveSwitchProvider(client, globalOpts, format) {
  const spinner = ora('Fetching providers...').start();

  try {
    const data = await client.listLLMProviders();
    spinner.stop();

    const providers = data.providers || [];
    const availableProviders = providers.filter(p => p.available);

    if (availableProviders.length === 0) {
      console.log(chalk.yellow('No available providers to switch to'));
      return;
    }

    const { selectedProvider } = await inquirer.prompt([
      {
        type: 'list',
        name: 'selectedProvider',
        message: 'Select LLM provider:',
        choices: availableProviders.map(p => ({
          name: `${p.name}${p.active ? ' (current)' : ''}`,
          value: p.name,
          disabled: p.active ? 'Already active' : false
        }))
      }
    ]);

    return await switchProvider(client, globalOpts, selectedProvider, format);
  } catch (error) {
    spinner.fail(chalk.red('Failed to fetch providers'));
    throw error;
  }
}

/**
 * Handle config subcommands (get/set)
 */
async function handleConfigCommand(client, globalOpts, args, format) {
  const action = args[0];

  switch (action) {
    case 'get':
      return getConfig(client, globalOpts, args[1], format);

    case 'set':
      return setConfig(client, globalOpts, args[1], args[2], format);

    default:
      console.log(chalk.red(`Unknown config command: ${action || '(none)'}`));
      console.log(chalk.gray('\nAvailable commands:'));
      console.log(chalk.gray('  config get [key]         Get LLM configuration'));
      console.log(chalk.gray('  config set <key> <value> Set LLM configuration'));
      process.exit(1);
  }
}

/**
 * Get LLM configuration
 */
async function getConfig(client, globalOpts, key, format) {
  const spinner = ora('Fetching LLM configuration...').start();

  try {
    const config = await client.getLLMConfig(key || null);
    spinner.stop();

    if (format === 'json' || globalOpts.json) {
      console.log(formatJSON(config));
    } else {
      console.log(chalk.blue.bold('\n⚙️  LLM Configuration\n'));

      if (key) {
        // Display specific key
        console.log(chalk.gray(`${key}: `) + chalk.white(config.value || 'N/A'));
      } else {
        // Display all config
        console.log(formatLLMConfig(config));
      }
    }
  } catch (error) {
    spinner.fail(chalk.red('Failed to fetch configuration'));
    throw error;
  }
}

/**
 * Set LLM configuration value
 */
async function setConfig(client, globalOpts, key, value, format) {
  if (!key || value === undefined) {
    console.log(chalk.red('Error: Key and value are required'));
    console.log(chalk.gray('Usage: riptide llm config set <key> <value>'));
    process.exit(1);
  }

  // Validate key
  if (!ALLOWED_CONFIG_KEYS.includes(key)) {
    console.log(chalk.yellow(`Warning: "${key}" is not a standard configuration key`));
    console.log(chalk.gray('Allowed keys: ' + ALLOWED_CONFIG_KEYS.join(', ')));

    const { proceed } = await inquirer.prompt([
      {
        type: 'confirm',
        name: 'proceed',
        message: 'Set this key anyway?',
        default: false
      }
    ]);

    if (!proceed) {
      console.log(chalk.yellow('Operation cancelled'));
      return;
    }
  }

  // Parse value (handle numbers and booleans)
  let parsedValue = value;
  if (value === 'true') parsedValue = true;
  else if (value === 'false') parsedValue = false;
  else if (!isNaN(value) && value !== '') parsedValue = parseFloat(value);

  const spinner = ora(`Setting ${key}...`).start();

  try {
    const result = await client.setLLMConfig(key, parsedValue);
    spinner.stop();

    if (format === 'json' || globalOpts.json) {
      console.log(formatJSON(result));
    } else {
      console.log(chalk.green(`✓ Set ${key} = ${parsedValue}`));

      if (result.requiresRestart) {
        console.log(chalk.yellow('\n⚠️  Configuration change requires service restart'));
      }
    }
  } catch (error) {
    spinner.fail(chalk.red(`Failed to set ${key}`));
    throw error;
  }
}

/**
 * Format provider switch result
 */
function formatSwitchResult(result) {
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
