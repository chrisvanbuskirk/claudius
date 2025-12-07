import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { ConfigManager } from '../../src/config/manager.js';
import { defaultInterests, defaultMcpServers, defaultPreferences } from '../../src/config/defaults.js';
import * as fs from 'fs';
import * as path from 'path';
import os from 'os';

describe('ConfigManager', () => {
  let manager: ConfigManager;
  let testConfigDir: string;

  beforeEach(() => {
    // Create a test config directory
    testConfigDir = path.join(os.tmpdir(), `claudius-config-test-${Date.now()}`);
    fs.mkdirSync(testConfigDir, { recursive: true });

    // Create a manager with mocked config directory
    manager = new ConfigManager();
    // Override the configDir using reflection
    (manager as unknown as { configDir: string }).configDir = testConfigDir;
  });

  afterEach(() => {
    // Clean up test directory
    if (fs.existsSync(testConfigDir)) {
      fs.rmSync(testConfigDir, { recursive: true });
    }
  });

  describe('ensureConfigDir', () => {
    it('should create config directory if it does not exist', () => {
      const newDir = path.join(testConfigDir, 'new-subdir');
      (manager as unknown as { configDir: string }).configDir = newDir;

      manager.ensureConfigDir();

      expect(fs.existsSync(newDir)).toBe(true);
    });

    it('should not throw if directory already exists', () => {
      expect(() => manager.ensureConfigDir()).not.toThrow();
    });
  });

  describe('getConfigPath', () => {
    it('should return correct path for config file', () => {
      const configPath = manager.getConfigPath('interests');

      expect(configPath).toBe(path.join(testConfigDir, 'interests.json'));
    });
  });

  describe('loadInterests', () => {
    it('should return defaults when no config file exists', () => {
      const interests = manager.loadInterests();

      expect(interests).toEqual(defaultInterests);
    });

    it('should load config from file', () => {
      const customConfig = {
        ...defaultInterests,
        topics: [{ name: 'Test Topic', enabled: true, keywords: [] }],
      };
      fs.writeFileSync(
        path.join(testConfigDir, 'interests.json'),
        JSON.stringify(customConfig)
      );

      const interests = manager.loadInterests();

      expect(interests.topics).toHaveLength(1);
      expect(interests.topics[0].name).toBe('Test Topic');
    });

    it('should merge with defaults for missing fields', () => {
      const partialConfig = { topics: [] };
      fs.writeFileSync(
        path.join(testConfigDir, 'interests.json'),
        JSON.stringify(partialConfig)
      );

      const interests = manager.loadInterests();

      expect(interests.keywords).toBeDefined();
      expect(interests.sources).toBeDefined();
      expect(interests.research_depth).toBeDefined();
    });

    it('should return defaults on parse error', () => {
      fs.writeFileSync(
        path.join(testConfigDir, 'interests.json'),
        'invalid json{'
      );

      const interests = manager.loadInterests();

      expect(interests).toEqual(defaultInterests);
    });
  });

  describe('saveInterests', () => {
    it('should save config to file', () => {
      const config = {
        ...defaultInterests,
        topics: [{ name: 'Saved Topic', enabled: true, keywords: ['test'] }],
      };

      manager.saveInterests(config);

      const saved = JSON.parse(
        fs.readFileSync(path.join(testConfigDir, 'interests.json'), 'utf-8')
      );
      expect(saved.topics[0].name).toBe('Saved Topic');
    });

    it('should create config directory if needed', () => {
      const newDir = path.join(testConfigDir, 'subdir');
      (manager as unknown as { configDir: string }).configDir = newDir;

      manager.saveInterests(defaultInterests);

      expect(fs.existsSync(path.join(newDir, 'interests.json'))).toBe(true);
    });
  });

  describe('loadMcpServers', () => {
    it('should return defaults when no config file exists', () => {
      const servers = manager.loadMcpServers();

      expect(servers).toEqual(defaultMcpServers);
    });

    it('should load config from file', () => {
      const customConfig = {
        servers: [
          {
            name: 'test-server',
            type: 'stdio',
            command: 'node',
            args: ['server.js'],
            enabled: true,
            config: {},
          },
        ],
      };
      fs.writeFileSync(
        path.join(testConfigDir, 'mcp-servers.json'),
        JSON.stringify(customConfig)
      );

      const servers = manager.loadMcpServers();

      expect(servers.servers).toHaveLength(1);
      expect(servers.servers[0].name).toBe('test-server');
    });
  });

  describe('saveMcpServers', () => {
    it('should save config to file', () => {
      const config = {
        servers: [
          {
            name: 'new-server',
            type: 'http' as const,
            command: 'python',
            args: [],
            enabled: false,
            config: { port: 8080 },
          },
        ],
      };

      manager.saveMcpServers(config);

      const saved = JSON.parse(
        fs.readFileSync(path.join(testConfigDir, 'mcp-servers.json'), 'utf-8')
      );
      expect(saved.servers[0].name).toBe('new-server');
    });
  });

  describe('loadPreferences', () => {
    it('should return defaults when no config file exists', () => {
      const prefs = manager.loadPreferences();

      expect(prefs).toEqual(defaultPreferences);
    });

    it('should load config from file', () => {
      const customConfig = {
        ...defaultPreferences,
        app: { ...defaultPreferences.app, theme: 'dark' },
      };
      fs.writeFileSync(
        path.join(testConfigDir, 'preferences.json'),
        JSON.stringify(customConfig)
      );

      const prefs = manager.loadPreferences();

      expect(prefs.app.theme).toBe('dark');
    });

    it('should merge nested objects with defaults', () => {
      const partialConfig = { app: { theme: 'light' } };
      fs.writeFileSync(
        path.join(testConfigDir, 'preferences.json'),
        JSON.stringify(partialConfig)
      );

      const prefs = manager.loadPreferences();

      expect(prefs.app.theme).toBe('light');
      expect(prefs.app.notifications).toBe(true); // From defaults
      expect(prefs.research.default_model).toBeDefined();
    });
  });

  describe('savePreferences', () => {
    it('should save config to file', () => {
      const config = {
        ...defaultPreferences,
        app: { ...defaultPreferences.app, notifications: false },
      };

      manager.savePreferences(config);

      const saved = JSON.parse(
        fs.readFileSync(path.join(testConfigDir, 'preferences.json'), 'utf-8')
      );
      expect(saved.app.notifications).toBe(false);
    });
  });

  describe('initializeAll', () => {
    it('should create all config files with defaults', () => {
      manager.initializeAll();

      expect(fs.existsSync(path.join(testConfigDir, 'interests.json'))).toBe(true);
      expect(fs.existsSync(path.join(testConfigDir, 'mcp-servers.json'))).toBe(true);
      expect(fs.existsSync(path.join(testConfigDir, 'preferences.json'))).toBe(true);
    });

    it('should not overwrite existing files', () => {
      const customInterests = { topics: [{ name: 'Custom', enabled: true, keywords: [] }] };
      fs.writeFileSync(
        path.join(testConfigDir, 'interests.json'),
        JSON.stringify(customInterests)
      );

      manager.initializeAll();

      const saved = JSON.parse(
        fs.readFileSync(path.join(testConfigDir, 'interests.json'), 'utf-8')
      );
      expect(saved.topics[0].name).toBe('Custom');
    });
  });

  describe('resetToDefaults', () => {
    it('should reset interests to defaults', () => {
      fs.writeFileSync(
        path.join(testConfigDir, 'interests.json'),
        JSON.stringify({ topics: [{ name: 'Old', enabled: true, keywords: [] }] })
      );

      manager.resetToDefaults('interests');

      const saved = JSON.parse(
        fs.readFileSync(path.join(testConfigDir, 'interests.json'), 'utf-8')
      );
      expect(saved).toEqual(defaultInterests);
    });

    it('should reset mcp-servers to defaults', () => {
      manager.resetToDefaults('mcp-servers');

      const saved = JSON.parse(
        fs.readFileSync(path.join(testConfigDir, 'mcp-servers.json'), 'utf-8')
      );
      expect(saved).toEqual(defaultMcpServers);
    });

    it('should reset preferences to defaults', () => {
      manager.resetToDefaults('preferences');

      const saved = JSON.parse(
        fs.readFileSync(path.join(testConfigDir, 'preferences.json'), 'utf-8')
      );
      expect(saved).toEqual(defaultPreferences);
    });
  });

  describe('validateInterests', () => {
    it('should return true for valid config', () => {
      expect(manager.validateInterests(defaultInterests)).toBe(true);
    });

    it('should return false for invalid config', () => {
      expect(manager.validateInterests(null)).toBe(false);
      expect(manager.validateInterests({})).toBe(false);
      expect(manager.validateInterests({ topics: 'not-array' })).toBe(false);
    });

    it('should validate research_depth values', () => {
      const validConfig = {
        ...defaultInterests,
        research_depth: { default: 'deep' as const },
      };
      const invalidConfig = {
        ...defaultInterests,
        research_depth: { default: 'invalid' },
      };

      expect(manager.validateInterests(validConfig)).toBe(true);
      expect(manager.validateInterests(invalidConfig)).toBe(false);
    });
  });

  describe('validateMcpServers', () => {
    it('should return true for valid config', () => {
      expect(manager.validateMcpServers(defaultMcpServers)).toBe(true);
    });

    it('should return false for invalid config', () => {
      expect(manager.validateMcpServers(null)).toBe(false);
      expect(manager.validateMcpServers({ servers: 'not-array' })).toBe(false);
    });

    it('should validate server structure', () => {
      const validConfig = {
        servers: [
          {
            name: 'test',
            type: 'stdio' as const,
            command: 'node',
            args: [],
            enabled: true,
            config: {},
          },
        ],
      };
      const invalidConfig = {
        servers: [{ name: 'test' }], // Missing required fields
      };

      expect(manager.validateMcpServers(validConfig)).toBe(true);
      expect(manager.validateMcpServers(invalidConfig)).toBe(false);
    });
  });

  describe('validatePreferences', () => {
    it('should return true for valid config', () => {
      expect(manager.validatePreferences(defaultPreferences)).toBe(true);
    });

    it('should return false for invalid config', () => {
      expect(manager.validatePreferences(null)).toBe(false);
      expect(manager.validatePreferences({})).toBe(false);
    });

    it('should validate theme values', () => {
      const validConfig = {
        ...defaultPreferences,
        app: { ...defaultPreferences.app, theme: 'dark' as const },
      };
      const invalidConfig = {
        ...defaultPreferences,
        app: { ...defaultPreferences.app, theme: 'invalid' },
      };

      expect(manager.validatePreferences(validConfig)).toBe(true);
      expect(manager.validatePreferences(invalidConfig)).toBe(false);
    });
  });
});
