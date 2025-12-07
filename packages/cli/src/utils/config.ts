import { join } from 'node:path';
import { homedir } from 'node:os';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';

/**
 * Get the Claudius configuration directory path
 * @returns Path to ~/.claudius
 */
export function getConfigDir(): string {
  return join(homedir(), '.claudius');
}

/**
 * Ensure the configuration directory exists
 */
export function ensureConfigDir(): void {
  const configDir = getConfigDir();
  if (!existsSync(configDir)) {
    mkdirSync(configDir, { recursive: true });
  }
}

/**
 * Load a JSON configuration file
 * @param name - Name of the config file (without .json extension)
 * @returns Parsed JSON data or null if file doesn't exist
 */
export function loadConfig(name: string): any {
  ensureConfigDir();
  const configPath = join(getConfigDir(), `${name}.json`);

  if (!existsSync(configPath)) {
    return null;
  }

  try {
    const data = readFileSync(configPath, 'utf-8');
    return JSON.parse(data);
  } catch (error) {
    throw new Error(`Failed to load config file ${name}: ${error}`);
  }
}

/**
 * Save a JSON configuration file
 * @param name - Name of the config file (without .json extension)
 * @param data - Data to save
 */
export function saveConfig(name: string, data: any): void {
  ensureConfigDir();
  const configPath = join(getConfigDir(), `${name}.json`);

  try {
    writeFileSync(configPath, JSON.stringify(data, null, 2), 'utf-8');
  } catch (error) {
    throw new Error(`Failed to save config file ${name}: ${error}`);
  }
}
