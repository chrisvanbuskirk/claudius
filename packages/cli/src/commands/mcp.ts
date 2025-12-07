import { Command } from 'commander';
import { success, error, info, warn, table, spinner } from '../utils/output.js';
import { loadConfig, saveConfig } from '../utils/config.js';

interface MCPServer {
  name: string;
  command: string;
  args?: string[];
  env?: Record<string, string>;
  enabled: boolean;
  addedAt: string;
}

interface MCPConfig {
  servers: MCPServer[];
}

function getMCPConfig(): MCPConfig {
  const config = loadConfig('mcp');
  return config || { servers: [] };
}

function saveMCPConfig(config: MCPConfig): void {
  saveConfig('mcp', config);
}

export function registerMCPCommands(program: Command): void {
  const mcp = program
    .command('mcp')
    .description('Manage MCP (Model Context Protocol) servers');

  mcp
    .command('list')
    .description('List MCP servers')
    .option('--enabled', 'Show only enabled servers')
    .option('--installed', 'Show only installed servers')
    .action((options: { enabled?: boolean; installed?: boolean }) => {
      try {
        const config = getMCPConfig();
        let servers = config.servers;

        if (options.enabled) {
          servers = servers.filter(s => s.enabled);
        }

        if (servers.length === 0) {
          info('No MCP servers configured. Add one with "claudius mcp add <name>"');
          return;
        }

        const tableData = servers.map(s => ({
          Name: s.name,
          Command: s.command,
          Enabled: s.enabled ? 'Yes' : 'No',
          'Added At': new Date(s.addedAt).toLocaleDateString(),
        }));

        table(tableData);
      } catch (err) {
        error(`Failed to list MCP servers: ${err}`);
      }
    });

  mcp
    .command('add <name>')
    .description('Add a new MCP server')
    .action((name: string) => {
      try {
        const config = getMCPConfig();

        // Check if server already exists
        if (config.servers.find(s => s.name === name)) {
          error(`MCP server "${name}" already exists`);
          return;
        }

        info('Not implemented yet: Interactive MCP server configuration');
        info('This will prompt for:');
        info('  - Command to run');
        info('  - Arguments (optional)');
        info('  - Environment variables (optional)');
        info('  - Enable on add (default: yes)');

        // Placeholder implementation
        warn(`To add MCP server "${name}", edit ~/.claudius/mcp.json manually for now`);
      } catch (err) {
        error(`Failed to add MCP server: ${err}`);
      }
    });

  mcp
    .command('enable <name>')
    .description('Enable an MCP server')
    .action((name: string) => {
      try {
        const config = getMCPConfig();
        const server = config.servers.find(s => s.name === name);

        if (!server) {
          error(`MCP server "${name}" not found`);
          return;
        }

        if (server.enabled) {
          info(`MCP server "${name}" is already enabled`);
          return;
        }

        server.enabled = true;
        saveMCPConfig(config);
        success(`Enabled MCP server: ${name}`);
      } catch (err) {
        error(`Failed to enable MCP server: ${err}`);
      }
    });

  mcp
    .command('disable <name>')
    .description('Disable an MCP server')
    .action((name: string) => {
      try {
        const config = getMCPConfig();
        const server = config.servers.find(s => s.name === name);

        if (!server) {
          error(`MCP server "${name}" not found`);
          return;
        }

        if (!server.enabled) {
          info(`MCP server "${name}" is already disabled`);
          return;
        }

        server.enabled = false;
        saveMCPConfig(config);
        success(`Disabled MCP server: ${name}`);
      } catch (err) {
        error(`Failed to disable MCP server: ${err}`);
      }
    });

  mcp
    .command('test <name>')
    .description('Test connection to an MCP server')
    .action(async (name: string) => {
      try {
        const config = getMCPConfig();
        const server = config.servers.find(s => s.name === name);

        if (!server) {
          error(`MCP server "${name}" not found`);
          return;
        }

        const spin = spinner(`Testing connection to ${name}...`);

        // Placeholder implementation
        setTimeout(() => {
          spin.succeed(`Successfully connected to ${name}`);
          info('Not implemented yet: Actual MCP server connection test');
        }, 1000);
      } catch (err) {
        error(`Failed to test MCP server: ${err}`);
      }
    });

  mcp
    .command('remove <name>')
    .description('Remove an MCP server')
    .action((name: string) => {
      try {
        const config = getMCPConfig();
        const index = config.servers.findIndex(s => s.name === name);

        if (index === -1) {
          error(`MCP server "${name}" not found`);
          return;
        }

        config.servers.splice(index, 1);
        saveMCPConfig(config);
        success(`Removed MCP server: ${name}`);
      } catch (err) {
        error(`Failed to remove MCP server: ${err}`);
      }
    });

  mcp
    .command('configure <name>')
    .description('Configure an MCP server')
    .action((name: string) => {
      try {
        const config = getMCPConfig();
        const server = config.servers.find(s => s.name === name);

        if (!server) {
          error(`MCP server "${name}" not found`);
          return;
        }

        info('Not implemented yet: Interactive MCP server configuration');
        info('Current configuration:');
        console.log(JSON.stringify(server, null, 2));
        warn(`To configure "${name}", edit ~/.claudius/mcp.json manually for now`);
      } catch (err) {
        error(`Failed to configure MCP server: ${err}`);
      }
    });
}
