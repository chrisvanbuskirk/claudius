import * as fs from 'fs';
import * as path from 'path';
import * as os from 'os';
import { InterestsConfig, MCPServersConfig, PreferencesConfig } from './types.js';
import { defaultInterests, defaultMcpServers, defaultPreferences } from './defaults.js';

export class ConfigManager {
  private configDir: string;

  constructor() {
    this.configDir = path.join(os.homedir(), '.claudius');
  }

  /**
   * Ensures the configuration directory exists
   */
  ensureConfigDir(): void {
    if (!fs.existsSync(this.configDir)) {
      fs.mkdirSync(this.configDir, { recursive: true });
    }
  }

  /**
   * Gets the full path for a configuration file
   */
  getConfigPath(name: string): string {
    return path.join(this.configDir, `${name}.json`);
  }

  /**
   * Loads interests configuration from file or returns defaults
   */
  loadInterests(): InterestsConfig {
    const configPath = this.getConfigPath('interests');

    try {
      if (fs.existsSync(configPath)) {
        const data = fs.readFileSync(configPath, 'utf-8');
        const parsed = JSON.parse(data);

        // Merge with defaults to ensure all required fields exist
        return {
          ...defaultInterests,
          ...parsed,
          keywords: {
            ...defaultInterests.keywords,
            ...parsed.keywords
          },
          sources: {
            ...defaultInterests.sources,
            ...parsed.sources
          },
          research_depth: {
            ...defaultInterests.research_depth,
            ...parsed.research_depth
          }
        };
      }
    } catch (error) {
      console.error(`Error loading interests config: ${error}`);
    }

    return { ...defaultInterests };
  }

  /**
   * Saves interests configuration to file
   */
  saveInterests(config: InterestsConfig): void {
    const configPath = this.getConfigPath('interests');

    try {
      this.ensureConfigDir();
      fs.writeFileSync(configPath, JSON.stringify(config, null, 2), 'utf-8');
    } catch (error) {
      throw new Error(`Failed to save interests config: ${error}`);
    }
  }

  /**
   * Loads MCP servers configuration from file or returns defaults
   */
  loadMcpServers(): MCPServersConfig {
    const configPath = this.getConfigPath('mcp-servers');

    try {
      if (fs.existsSync(configPath)) {
        const data = fs.readFileSync(configPath, 'utf-8');
        const parsed = JSON.parse(data);

        // Merge with defaults
        return {
          ...defaultMcpServers,
          ...parsed
        };
      }
    } catch (error) {
      console.error(`Error loading MCP servers config: ${error}`);
    }

    return { ...defaultMcpServers };
  }

  /**
   * Saves MCP servers configuration to file
   */
  saveMcpServers(config: MCPServersConfig): void {
    const configPath = this.getConfigPath('mcp-servers');

    try {
      this.ensureConfigDir();
      fs.writeFileSync(configPath, JSON.stringify(config, null, 2), 'utf-8');
    } catch (error) {
      throw new Error(`Failed to save MCP servers config: ${error}`);
    }
  }

  /**
   * Loads preferences configuration from file or returns defaults
   */
  loadPreferences(): PreferencesConfig {
    const configPath = this.getConfigPath('preferences');

    try {
      if (fs.existsSync(configPath)) {
        const data = fs.readFileSync(configPath, 'utf-8');
        const parsed = JSON.parse(data);

        // Merge with defaults to ensure all required fields exist
        return {
          ...defaultPreferences,
          ...parsed,
          app: {
            ...defaultPreferences.app,
            ...parsed.app
          },
          research: {
            ...defaultPreferences.research,
            ...parsed.research
          },
          storage: {
            ...defaultPreferences.storage,
            ...parsed.storage
          }
        };
      }
    } catch (error) {
      console.error(`Error loading preferences config: ${error}`);
    }

    return { ...defaultPreferences };
  }

  /**
   * Saves preferences configuration to file
   */
  savePreferences(config: PreferencesConfig): void {
    const configPath = this.getConfigPath('preferences');

    try {
      this.ensureConfigDir();
      fs.writeFileSync(configPath, JSON.stringify(config, null, 2), 'utf-8');
    } catch (error) {
      throw new Error(`Failed to save preferences config: ${error}`);
    }
  }

  /**
   * Initializes all configuration files with defaults if they don't exist
   */
  initializeAll(): void {
    this.ensureConfigDir();

    // Initialize interests config if it doesn't exist
    const interestsPath = this.getConfigPath('interests');
    if (!fs.existsSync(interestsPath)) {
      try {
        this.saveInterests(defaultInterests);
        console.log(`Created default interests config at ${interestsPath}`);
      } catch (error) {
        console.error(`Failed to create interests config: ${error}`);
      }
    }

    // Initialize MCP servers config if it doesn't exist
    const mcpServersPath = this.getConfigPath('mcp-servers');
    if (!fs.existsSync(mcpServersPath)) {
      try {
        this.saveMcpServers(defaultMcpServers);
        console.log(`Created default MCP servers config at ${mcpServersPath}`);
      } catch (error) {
        console.error(`Failed to create MCP servers config: ${error}`);
      }
    }

    // Initialize preferences config if it doesn't exist
    const preferencesPath = this.getConfigPath('preferences');
    if (!fs.existsSync(preferencesPath)) {
      try {
        this.savePreferences(defaultPreferences);
        console.log(`Created default preferences config at ${preferencesPath}`);
      } catch (error) {
        console.error(`Failed to create preferences config: ${error}`);
      }
    }
  }

  /**
   * Resets a specific configuration to defaults
   */
  resetToDefaults(configType: 'interests' | 'mcp-servers' | 'preferences'): void {
    try {
      switch (configType) {
        case 'interests':
          this.saveInterests(defaultInterests);
          break;
        case 'mcp-servers':
          this.saveMcpServers(defaultMcpServers);
          break;
        case 'preferences':
          this.savePreferences(defaultPreferences);
          break;
      }
      console.log(`Reset ${configType} configuration to defaults`);
    } catch (error) {
      throw new Error(`Failed to reset ${configType} config: ${error}`);
    }
  }

  /**
   * Validates that a configuration object matches the expected structure
   */
  validateInterests(config: unknown): config is InterestsConfig {
    const c = config as InterestsConfig;
    return (
      typeof c === 'object' &&
      c !== null &&
      Array.isArray(c.topics) &&
      typeof c.keywords === 'object' &&
      Array.isArray(c.keywords.include) &&
      Array.isArray(c.keywords.exclude) &&
      typeof c.sources === 'object' &&
      Array.isArray(c.sources.github) &&
      Array.isArray(c.sources.rss) &&
      typeof c.research_depth === 'object' &&
      (c.research_depth.default === 'surface' ||
        c.research_depth.default === 'balanced' ||
        c.research_depth.default === 'deep')
    );
  }

  /**
   * Validates MCP servers configuration
   */
  validateMcpServers(config: unknown): config is MCPServersConfig {
    const c = config as MCPServersConfig;
    return (
      typeof c === 'object' &&
      c !== null &&
      Array.isArray(c.servers) &&
      c.servers.every(
        (server) =>
          typeof server === 'object' &&
          server !== null &&
          typeof server.name === 'string' &&
          (server.type === 'stdio' || server.type === 'http') &&
          typeof server.command === 'string' &&
          typeof server.enabled === 'boolean' &&
          typeof server.config === 'object'
      )
    );
  }

  /**
   * Validates preferences configuration
   */
  validatePreferences(config: unknown): config is PreferencesConfig {
    const c = config as PreferencesConfig;
    return (
      typeof c === 'object' &&
      c !== null &&
      typeof c.app === 'object' &&
      (c.app.theme === 'light' ||
        c.app.theme === 'dark' ||
        c.app.theme === 'system') &&
      typeof c.app.notifications === 'boolean' &&
      typeof c.research === 'object' &&
      typeof c.research.schedule === 'string' &&
      typeof c.research.default_model === 'string' &&
      typeof c.research.max_tokens === 'number' &&
      typeof c.research.temperature === 'number' &&
      typeof c.research.num_briefings === 'number' &&
      typeof c.storage === 'object' &&
      typeof c.storage.db_path === 'string' &&
      typeof c.storage.briefing_retention_days === 'number'
    );
  }
}

export const configManager = new ConfigManager();
