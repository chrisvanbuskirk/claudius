import chalk from 'chalk';
import ora, { Ora } from 'ora';

/**
 * Print success message in green
 */
export function success(msg: string): void {
  console.log(chalk.green('✓'), msg);
}

/**
 * Print error message in red
 */
export function error(msg: string): void {
  console.error(chalk.red('✗'), msg);
}

/**
 * Print info message in blue
 */
export function info(msg: string): void {
  console.log(chalk.blue('ℹ'), msg);
}

/**
 * Print warning message in yellow
 */
export function warn(msg: string): void {
  console.log(chalk.yellow('⚠'), msg);
}

/**
 * Format data as a table
 * @param data - Array of objects to display as table
 */
export function table(data: Record<string, any>[]): void {
  if (data.length === 0) {
    info('No data to display');
    return;
  }

  // Get all unique keys
  const keys = Array.from(new Set(data.flatMap(obj => Object.keys(obj))));

  // Calculate column widths
  const widths: Record<string, number> = {};
  keys.forEach(key => {
    widths[key] = Math.max(
      key.length,
      ...data.map(row => String(row[key] || '').length)
    );
  });

  // Print header
  const header = keys.map(key => key.padEnd(widths[key])).join(' | ');
  console.log(chalk.bold(header));
  console.log(keys.map(key => '-'.repeat(widths[key])).join('-+-'));

  // Print rows
  data.forEach(row => {
    const line = keys.map(key => String(row[key] || '').padEnd(widths[key])).join(' | ');
    console.log(line);
  });
}

/**
 * Create a spinner for long-running operations
 * @param msg - Message to display
 * @returns Ora spinner instance
 */
export function spinner(msg: string): Ora {
  return ora(msg).start();
}
