import { Command } from 'commander';
import { success, error, info, warn } from '../utils/output.js';
import { getConfigDir, loadConfig, saveConfig } from '../utils/config.js';
import { execSync as _execSync } from 'node:child_process';
import { join } from 'node:path';
// Note: execSync is available for future editor integration
void _execSync;
import { existsSync, readFileSync } from 'node:fs';

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

  // API Key management commands - using file-based storage in ~/.claudius/.env
  config
    .command('api-key')
    .description('Show API key status (stored in ~/.claudius/.env)')
    .action(() => {
      try {
        const configDir = getConfigDir();
        const envPath = join(configDir, '.env');

        if (existsSync(envPath)) {
          const content = readFileSync(envPath, 'utf-8');
          const keyLine = content.split('\n').find(line => line.startsWith('ANTHROPIC_API_KEY='));
          if (keyLine) {
            const key = keyLine.replace('ANTHROPIC_API_KEY=', '').trim().replace(/["']/g, '');
            if (key.length > 12) {
              const masked = `${key.slice(0, 8)}...${key.slice(-4)}`;
              success(`API key configured: ${masked}`);
            } else if (key) {
              success('API key configured: ****');
            } else {
              warn('No API key configured');
              info('Set your API key with: claudius config api-key set <your-key>');
            }
            return;
          }
        }

        warn('No API key configured');
        info('Set your API key with: claudius config api-key set <your-key>');
      } catch (err) {
        error(`Failed to check API key: ${err}`);
      }
    });

  config
    .command('api-key set <key>')
    .description('Set your Anthropic API key (stored in ~/.claudius/.env)')
    .action((key: string) => {
      try {
        if (!key.startsWith('sk-ant-')) {
          error("Invalid API key format. Anthropic API keys start with 'sk-ant-'");
          return;
        }

        const configDir = getConfigDir();
        const envPath = join(configDir, '.env');

        // Read existing content to preserve other variables
        let lines: string[] = [];
        let keyUpdated = false;

        if (existsSync(envPath)) {
          const content = readFileSync(envPath, 'utf-8');
          for (const line of content.split('\n')) {
            if (line.trim().startsWith('ANTHROPIC_API_KEY=')) {
              lines.push(`ANTHROPIC_API_KEY=${key}`);
              keyUpdated = true;
            } else if (line.trim()) {
              lines.push(line);
            }
          }
        }

        if (!keyUpdated) {
          lines.push(`ANTHROPIC_API_KEY=${key}`);
        }

        const { writeFileSync, mkdirSync } = require('node:fs');
        mkdirSync(configDir, { recursive: true });
        writeFileSync(envPath, lines.join('\n') + '\n', { mode: 0o600 });

        success('API key saved to ~/.claudius/.env');
        info('This key is used by both the CLI and desktop app.');
      } catch (err) {
        error(`Failed to set API key: ${err}`);
      }
    });

  config
    .command('api-key clear')
    .description('Remove your Anthropic API key')
    .action(() => {
      try {
        const configDir = getConfigDir();
        const envPath = join(configDir, '.env');

        if (!existsSync(envPath)) {
          success('API key already cleared');
          return;
        }

        const content = readFileSync(envPath, 'utf-8');
        const lines = content
          .split('\n')
          .filter(line => !line.trim().startsWith('ANTHROPIC_API_KEY='));

        if (lines.filter(l => l.trim()).length === 0) {
          const { unlinkSync } = require('node:fs');
          unlinkSync(envPath);
        } else {
          const { writeFileSync } = require('node:fs');
          writeFileSync(envPath, lines.join('\n') + '\n', { mode: 0o600 });
        }

        success('API key cleared');
      } catch (err) {
        error(`Failed to clear API key: ${err}`);
      }
    });
}
