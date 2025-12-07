import { Command } from 'commander';
import { success, error, info, warn } from '../utils/output.js';
import { getConfigDir, loadConfig, saveConfig } from '../utils/config.js';
import { execSync as _execSync } from 'node:child_process';
import { join as _join } from 'node:path';
// Note: execSync and join are available for future editor integration
void _execSync; void _join;
import { existsSync } from 'node:fs';

export function registerConfigCommands(program: Command): void {
  const config = program
    .command('config')
    .description('Manage Claudius configuration');

  config
    .command('show')
    .description('Show current configuration')
    .action(() => {
      try {
        const configDir = getConfigDir();

        if (!existsSync(configDir)) {
          warn('Configuration directory does not exist. Run "claudius config init" first.');
          return;
        }

        info(`Configuration directory: ${configDir}`);
        console.log();

        // Load and display all config files
        const configFiles = ['interests', 'mcp', 'settings'];

        configFiles.forEach(name => {
          const data = loadConfig(name);
          if (data) {
            console.log(`${name}.json:`);
            console.log(JSON.stringify(data, null, 2));
            console.log();
          }
        });
      } catch (err) {
        error(`Failed to show configuration: ${err}`);
      }
    });

  config
    .command('edit')
    .description('Open configuration directory in default editor')
    .action(() => {
      try {
        const configDir = getConfigDir();

        if (!existsSync(configDir)) {
          warn('Configuration directory does not exist. Run "claudius config init" first.');
          return;
        }

        const editor = process.env.EDITOR || process.env.VISUAL || 'vim';

        info(`Opening ${configDir} in ${editor}...`);
        info('Not implemented yet: Interactive editor opening');
        info(`You can manually edit files at: ${configDir}`);
      } catch (err) {
        error(`Failed to open editor: ${err}`);
      }
    });

  config
    .command('set <key> <value>')
    .description('Set a configuration value')
    .action((key: string, value: string) => {
      try {
        // Parse key as file.path.to.value
        const parts = key.split('.');
        if (parts.length < 2) {
          error('Key must be in format: <file>.<key> (e.g., settings.model)');
          return;
        }

        const [configName, ...keyPath] = parts;
        let data = loadConfig(configName) || {};

        // Navigate to the nested key
        let current = data;
        for (let i = 0; i < keyPath.length - 1; i++) {
          if (!current[keyPath[i]]) {
            current[keyPath[i]] = {};
          }
          current = current[keyPath[i]];
        }

        // Try to parse value as JSON, fallback to string
        let parsedValue: any = value;
        try {
          parsedValue = JSON.parse(value);
        } catch {
          // Keep as string
        }

        // Set the value
        current[keyPath[keyPath.length - 1]] = parsedValue;

        saveConfig(configName, data);
        success(`Set ${key} = ${parsedValue}`);
      } catch (err) {
        error(`Failed to set configuration: ${err}`);
      }
    });

  config
    .command('init')
    .description('Initialize configuration files')
    .action(() => {
      try {
        const configDir = getConfigDir();

        if (existsSync(configDir)) {
          warn('Configuration directory already exists');
          info(`Location: ${configDir}`);
        } else {
          // ensureConfigDir is called automatically by saveConfig
          success(`Created configuration directory: ${configDir}`);
        }

        // Create default config files
        const interests = loadConfig('interests') || { interests: [], blocked: [] };
        saveConfig('interests', interests);

        const mcp = loadConfig('mcp') || { servers: [] };
        saveConfig('mcp', mcp);

        const settings = loadConfig('settings') || {
          model: 'claude-opus-4',
          defaultDepth: 3,
          scheduleInterval: '0 8 * * *', // Daily at 8 AM
          outputFormat: 'markdown',
        };
        saveConfig('settings', settings);

        success('Initialized configuration files');
        info(`Location: ${configDir}`);
      } catch (err) {
        error(`Failed to initialize configuration: ${err}`);
      }
    });
}
