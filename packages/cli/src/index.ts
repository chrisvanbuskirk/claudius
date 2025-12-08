#!/usr/bin/env node

import { program } from 'commander';
import { readFileSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

// Import all command modules
import { registerInterestsCommands } from './commands/interests.js';
import { registerMCPCommands } from './commands/mcp.js';
import { registerResearchCommands } from './commands/research.js';
import { registerBriefingsCommands } from './commands/briefings.js';
import { registerConfigCommands } from './commands/config.js';
import { registerDBCommands } from './commands/db.js';
import { registerSchedulerCommands } from './commands/scheduler.js';

// Get package.json for version
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const packageJson = JSON.parse(
  readFileSync(join(__dirname, '..', 'package.json'), 'utf-8')
);

// Set up program with version and description
program
  .name('claudius')
  .description('CLI for Claudius - AI-powered personalized research briefing system')
  .version(packageJson.version);

// Register all commands
registerInterestsCommands(program);
registerMCPCommands(program);
registerResearchCommands(program);
registerBriefingsCommands(program);
registerConfigCommands(program);
registerDBCommands(program);
registerSchedulerCommands(program);

// Parse command line arguments
program.parse();
