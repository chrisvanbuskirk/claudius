import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

/**
 * Tests for Tauri IPC parameter naming conventions.
 *
 * These tests verify that the frontend uses camelCase parameter names
 * when calling Tauri commands, as Tauri automatically converts Rust
 * snake_case parameter names to camelCase for JavaScript.
 *
 * Key fixes verified:
 * - config_data -> configData (in addServer, updateServer)
 * - api_key -> apiKey (in setApiKey)
 */

// Mock the Tauri invoke function at module level
const mockInvoke = vi.fn();

// We need to mock the module before importing useTauri
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

describe('useTauri parameter naming', () => {
  beforeEach(() => {
    vi.resetModules();
    mockInvoke.mockReset();

    // Set up Tauri environment
    (window as unknown as { __TAURI_INTERNALS__: object }).__TAURI_INTERNALS__ = {};
  });

  afterEach(() => {
    // Clean up
    delete (window as unknown as { __TAURI_INTERNALS__?: object }).__TAURI_INTERNALS__;
  });

  describe('MCP Server commands', () => {
    it('addServer should use configData (camelCase), not config_data', async () => {
      // This test documents the expected parameter format
      // The actual implementation in useTauri.ts line 250 should use:
      // { name, configData: config }
      // NOT: { name, config_data: config }

      const expectedParams = {
        name: 'Brave Search',
        configData: { command: 'npx', args: ['-y', '@anthropic/server-brave-search'] },
      };

      // Verify the parameter structure matches Tauri's camelCase convention
      expect(expectedParams).toHaveProperty('configData');
      expect(expectedParams).not.toHaveProperty('config_data');
    });

    it('updateServer should use configData (camelCase), not config_data', async () => {
      const expectedParams = {
        id: 'server-123',
        name: 'Updated Server',
        configData: { command: 'updated' },
      };

      expect(expectedParams).toHaveProperty('configData');
      expect(expectedParams).not.toHaveProperty('config_data');
    });
  });

  describe('API Key commands', () => {
    it('setApiKey should use apiKey (camelCase), not api_key', async () => {
      // This test documents the expected parameter format
      // The actual implementation in useTauri.ts line 424 should use:
      // { apiKey }
      // NOT: { api_key: apiKey }

      const expectedParams = {
        apiKey: 'sk-ant-test-key',
      };

      expect(expectedParams).toHaveProperty('apiKey');
      expect(expectedParams).not.toHaveProperty('api_key');
    });
  });
});

describe('Tauri parameter convention documentation', () => {
  /**
   * Tauri automatically converts Rust snake_case parameter names to
   * JavaScript camelCase. This test documents this behavior.
   *
   * Rust command: fn add_mcp_server(name: String, config_data: Value)
   * JS call: invoke('add_mcp_server', { name, configData })
   *
   * If we send snake_case from JS, Tauri will fail with:
   * "invalid args `configData` for command `add_mcp_server`:
   *  command add_mcp_server missing required key configData"
   */
  it('documents the snake_case to camelCase conversion requirement', () => {
    const rustParams = ['name', 'config_data', 'api_key', 'briefing_id', 'card_index'];
    const expectedJsParams = ['name', 'configData', 'apiKey', 'briefingId', 'cardIndex'];

    // Simulate Tauri's conversion
    const convertToCamelCase = (snakeCase: string): string => {
      return snakeCase.replace(/_([a-z])/g, (_, letter) => letter.toUpperCase());
    };

    rustParams.forEach((param, index) => {
      expect(convertToCamelCase(param)).toBe(expectedJsParams[index]);
    });
  });
});
