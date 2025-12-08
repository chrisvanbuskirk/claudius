import { Command } from 'commander';
import { writeFileSync, readFileSync, existsSync, unlinkSync } from 'node:fs';
import { join } from 'node:path';
import { execSync } from 'node:child_process';
import { homedir } from 'node:os';
import { configManager } from '@claudius/shared';

const PLIST_NAME = 'com.claudius.research';
const PLIST_FILE = `${PLIST_NAME}.plist`;

function getLaunchAgentsDir(): string {
  return join(homedir(), 'Library', 'LaunchAgents');
}

function getPlistPath(): string {
  return join(getLaunchAgentsDir(), PLIST_FILE);
}

function getCliPath(): string {
  // In production, this would be the installed CLI path
  // For development, use the local path
  const globalPath = '/usr/local/bin/claudius';
  if (existsSync(globalPath)) {
    return globalPath;
  }
  // Try npx fallback
  return 'npx';
}

function generatePlist(hour: number, minute: number): string {
  const cliPath = getCliPath();
  const isNpx = cliPath === 'npx';

  const programArgs = isNpx
    ? `    <string>npx</string>
    <string>@claudius/cli</string>
    <string>research</string>
    <string>--now</string>`
    : `    <string>${cliPath}</string>
    <string>research</string>
    <string>--now</string>`;

  return `<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>${PLIST_NAME}</string>

  <key>ProgramArguments</key>
  <array>
${programArgs}
  </array>

  <key>StartCalendarInterval</key>
  <dict>
    <key>Hour</key>
    <integer>${hour}</integer>
    <key>Minute</key>
    <integer>${minute}</integer>
  </dict>

  <key>StandardOutPath</key>
  <string>${homedir()}/.claudius/logs/research.log</string>

  <key>StandardErrorPath</key>
  <string>${homedir()}/.claudius/logs/research-error.log</string>

  <key>EnvironmentVariables</key>
  <dict>
    <key>PATH</key>
    <string>/usr/local/bin:/usr/bin:/bin:/opt/homebrew/bin</string>
  </dict>

  <key>RunAtLoad</key>
  <false/>
</dict>
</plist>`;
}

export function registerSchedulerCommands(program: Command): void {
  const scheduler = program
    .command('scheduler')
    .description('Manage background research scheduling');

  scheduler
    .command('install')
    .description('Install launchd service for background research scheduling')
    .option('-t, --time <time>', 'Time to run research (HH:MM format)', '06:00')
    .action(async (options) => {
      try {
        configManager.initializeAll();

        // Parse time
        const [hourStr, minuteStr] = options.time.split(':');
        const hour = parseInt(hourStr, 10);
        const minute = parseInt(minuteStr, 10);

        if (isNaN(hour) || isNaN(minute) || hour < 0 || hour > 23 || minute < 0 || minute > 59) {
          console.error(`Invalid time format: ${options.time}. Use HH:MM (e.g., 06:00)`);
          process.exit(1);
        }

        // Create logs directory
        const logsDir = join(homedir(), '.claudius', 'logs');
        execSync(`mkdir -p "${logsDir}"`);

        // Generate and write plist
        const plist = generatePlist(hour, minute);
        const plistPath = getPlistPath();

        // Unload existing service if present
        try {
          execSync(`launchctl unload "${plistPath}" 2>/dev/null`, { stdio: 'ignore' });
        } catch {
          // Ignore errors if not loaded
        }

        writeFileSync(plistPath, plist);
        console.log(`Created launchd plist at: ${plistPath}`);

        // Load the service
        execSync(`launchctl load "${plistPath}"`);
        console.log(`Loaded launchd service: ${PLIST_NAME}`);

        // Update preferences with cron expression
        const prefs = configManager.loadPreferences();
        prefs.research.schedule = `${minute} ${hour} * * *`;
        configManager.savePreferences(prefs);

        console.log(`\nResearch scheduled for ${hour.toString().padStart(2, '0')}:${minute.toString().padStart(2, '0')} daily`);
        console.log('The service will run automatically even when the app is not open.');
      } catch (error) {
        console.error('Failed to install scheduler:', error);
        process.exit(1);
      }
    });

  scheduler
    .command('uninstall')
    .description('Uninstall launchd service')
    .action(async () => {
      try {
        const plistPath = getPlistPath();

        if (!existsSync(plistPath)) {
          console.log('Scheduler is not installed.');
          return;
        }

        // Unload the service
        try {
          execSync(`launchctl unload "${plistPath}"`);
          console.log(`Unloaded launchd service: ${PLIST_NAME}`);
        } catch {
          console.log('Service was not loaded.');
        }

        // Remove plist file
        unlinkSync(plistPath);
        console.log(`Removed plist file: ${plistPath}`);

        console.log('\nBackground scheduling disabled.');
        console.log('Note: The app will still schedule research when open.');
      } catch (error) {
        console.error('Failed to uninstall scheduler:', error);
        process.exit(1);
      }
    });

  scheduler
    .command('status')
    .description('Show scheduler status')
    .action(async () => {
      try {
        configManager.initializeAll();
        const prefs = configManager.loadPreferences();

        console.log('Scheduler Status\n');

        // Check launchd service
        const plistPath = getPlistPath();
        const isInstalled = existsSync(plistPath);

        console.log(`launchd service: ${isInstalled ? 'Installed' : 'Not installed'}`);

        if (isInstalled) {
          try {
            const output = execSync(`launchctl list | grep ${PLIST_NAME}`, { encoding: 'utf-8' });
            const isLoaded = output.includes(PLIST_NAME);
            console.log(`Service loaded: ${isLoaded ? 'Yes' : 'No'}`);

            // Read plist to show scheduled time
            const plistContent = readFileSync(plistPath, 'utf-8');
            const hourMatch = plistContent.match(/<key>Hour<\/key>\s*<integer>(\d+)<\/integer>/);
            const minuteMatch = plistContent.match(/<key>Minute<\/key>\s*<integer>(\d+)<\/integer>/);

            if (hourMatch && minuteMatch) {
              const hour = hourMatch[1].padStart(2, '0');
              const minute = minuteMatch[1].padStart(2, '0');
              console.log(`Scheduled time: ${hour}:${minute} daily`);
            }
          } catch {
            console.log('Service loaded: No');
          }
        }

        console.log(`\nCron schedule (in-app): ${prefs.research.schedule || 'Not set'}`);

        // Show logs location
        const logsDir = join(homedir(), '.claudius', 'logs');
        console.log(`\nLogs directory: ${logsDir}`);
      } catch (error) {
        console.error('Failed to get status:', error);
        process.exit(1);
      }
    });

  scheduler
    .command('logs')
    .description('Show recent scheduler logs')
    .option('-n, --lines <number>', 'Number of lines to show', '50')
    .option('-e, --errors', 'Show error log instead of output log')
    .action((options) => {
      const logsDir = join(homedir(), '.claudius', 'logs');
      const logFile = options.errors ? 'research-error.log' : 'research.log';
      const logPath = join(logsDir, logFile);

      if (!existsSync(logPath)) {
        console.log(`No log file found at: ${logPath}`);
        console.log('Research has not run yet or logs were cleared.');
        return;
      }

      try {
        const output = execSync(`tail -n ${options.lines} "${logPath}"`, { encoding: 'utf-8' });
        console.log(`Last ${options.lines} lines of ${logFile}:\n`);
        console.log(output);
      } catch (error) {
        console.error('Failed to read logs:', error);
      }
    });

  scheduler
    .command('run')
    .description('Manually trigger research now')
    .action(() => {
      console.log('Research is now handled by the Claudius desktop app.\n');
      console.log('To run research:');
      console.log('  1. Open the Claudius desktop app');
      console.log('  2. Click "Run Research Now" or use the popover menu');
      console.log('\nThe desktop app includes a built-in Rust research agent');
      console.log('that runs automatically on your configured schedule.');
    });
}
