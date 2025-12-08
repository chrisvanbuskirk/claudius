import { Command } from 'commander';
import { error, info, warn, table, spinner } from '../utils/output.js';
import {
  initDatabase,
  getAllBriefings,
  searchBriefings,
  getBriefing,
  deleteBriefing,
  getBriefingsByDate,
} from '@claudius/shared';
import * as fs from 'fs';
import * as path from 'path';
import * as os from 'os';

function getDbPath(): string {
  return path.join(os.homedir(), '.claudius', 'claudius.db');
}

async function ensureDb(): Promise<void> {
  await initDatabase(getDbPath());
}

export function registerBriefingsCommands(program: Command): void {
  const briefings = program
    .command('briefings')
    .description('Query and manage research briefings');

  briefings
    .command('list')
    .description('List all briefings')
    .option('--last <days>', 'Show briefings from last N days', '30')
    .option('--limit <count>', 'Maximum number of briefings to show', '50')
    .action(async (options: { last: string; limit: string }) => {
      try {
        await ensureDb();
        const limit = parseInt(options.limit, 10);
        const days = parseInt(options.last, 10);

        // Calculate date range
        const endDate = new Date();
        const startDate = new Date();
        startDate.setDate(startDate.getDate() - days);

        const startStr = startDate.toISOString().split('T')[0];
        const endStr = endDate.toISOString().split('T')[0];

        const results = getBriefingsByDate(startStr, endStr);

        if (results.length === 0) {
          info(`No briefings found in the last ${days} days.`);
          info('Run "claudius research --now" to generate new briefings.');
          return;
        }

        const tableData = results.slice(0, limit).map((b) => ({
          ID: b.id,
          Date: b.date,
          Title: b.title || '(untitled)',
          Cards: b.cards.length,
          Model: b.model_used || '-',
        }));

        info(`Found ${results.length} briefing(s) in the last ${days} days:\n`);
        table(tableData);
      } catch (err) {
        error(`Failed to list briefings: ${err}`);
      }
    });

  briefings
    .command('search <query>')
    .description('Search briefings by content')
    .option('--since <date>', 'Only search briefings since this date (YYYY-MM-DD)')
    .action(async (query: string, options: { since?: string }) => {
      try {
        await ensureDb();

        const results = searchBriefings(query);

        // Filter by date if specified
        let filtered = results;
        if (options.since) {
          const sinceDate = new Date(options.since);
          filtered = results.filter((b) => new Date(b.date) >= sinceDate);
        }

        if (filtered.length === 0) {
          info(`No briefings found matching "${query}".`);
          return;
        }

        info(`Found ${filtered.length} briefing(s) matching "${query}":\n`);

        const tableData = filtered.map((b) => ({
          ID: b.id,
          Date: b.date,
          Title: b.title || '(untitled)',
          Cards: b.cards.length,
        }));

        table(tableData);
      } catch (err) {
        error(`Failed to search briefings: ${err}`);
      }
    });

  briefings
    .command('show <id>')
    .description('Show details of a specific briefing')
    .action(async (id: string) => {
      try {
        await ensureDb();
        const briefingId = parseInt(id, 10);

        if (isNaN(briefingId)) {
          error('Invalid briefing ID. Please provide a number.');
          return;
        }

        const briefing = getBriefing(briefingId);

        if (!briefing) {
          error(`Briefing with ID ${id} not found.`);
          return;
        }

        info(`\n=== Briefing #${briefing.id} ===`);
        info(`Date: ${briefing.date}`);
        info(`Title: ${briefing.title || '(untitled)'}`);
        info(`Model: ${briefing.model_used || 'unknown'}`);
        if (briefing.research_time_ms) {
          info(`Research time: ${(briefing.research_time_ms / 1000).toFixed(1)}s`);
        }
        if (briefing.total_tokens) {
          info(`Total tokens: ${briefing.total_tokens}`);
        }

        info(`\n--- Cards (${briefing.cards.length}) ---`);

        briefing.cards.forEach((card, i) => {
          info(`\n[${i + 1}] ${card.title}`);
          info(`    ${card.summary}`);
          if (card.sources && card.sources.length > 0) {
            info(`    Sources: ${card.sources.map((s) => s.url || s).join(', ')}`);
          }
        });
      } catch (err) {
        error(`Failed to show briefing: ${err}`);
      }
    });

  briefings
    .command('export <id>')
    .description('Export a briefing')
    .option('--format <format>', 'Export format (markdown, json)', 'markdown')
    .option('--output <path>', 'Output file path')
    .action(async (id: string, options: { format: string; output?: string }) => {
      try {
        await ensureDb();
        const validFormats = ['markdown', 'json'];
        if (!validFormats.includes(options.format)) {
          error(`Invalid format. Choose from: ${validFormats.join(', ')}`);
          return;
        }

        const spin = spinner(`Exporting briefing ${id}...`);
        const briefingId = parseInt(id, 10);

        if (isNaN(briefingId)) {
          spin.fail('Invalid briefing ID');
          return;
        }

        const briefing = getBriefing(briefingId);
        if (!briefing) {
          spin.fail(`Briefing with ID ${id} not found`);
          return;
        }

        let content: string;
        let ext: string;

        if (options.format === 'json') {
          content = JSON.stringify(briefing, null, 2);
          ext = 'json';
        } else {
          // Markdown format
          const lines: string[] = [
            `# ${briefing.title || 'Research Briefing'}`,
            '',
            `**Date:** ${briefing.date}`,
            `**Model:** ${briefing.model_used || 'unknown'}`,
            '',
            '---',
            '',
          ];

          briefing.cards.forEach((card, i) => {
            lines.push(`## ${i + 1}. ${card.title}`);
            lines.push('');
            lines.push(card.summary);
            lines.push('');
            if (card.sources && card.sources.length > 0) {
              lines.push('### Sources');
              card.sources.forEach((src) => {
                const url = typeof src === 'string' ? src : src.url;
                lines.push(`- ${url}`);
              });
              lines.push('');
            }
          });

          content = lines.join('\n');
          ext = 'md';
        }

        const outputPath = options.output || `briefing-${id}.${ext}`;
        fs.writeFileSync(outputPath, content);

        spin.succeed(`Exported briefing to ${outputPath}`);
      } catch (err) {
        error(`Failed to export briefing: ${err}`);
      }
    });

  briefings
    .command('delete <id>')
    .description('Delete a briefing')
    .option('--force', 'Skip confirmation', false)
    .action(async (id: string, options: { force: boolean }) => {
      try {
        await ensureDb();
        const briefingId = parseInt(id, 10);

        if (isNaN(briefingId)) {
          error('Invalid briefing ID. Please provide a number.');
          return;
        }

        const briefing = getBriefing(briefingId);
        if (!briefing) {
          error(`Briefing with ID ${id} not found.`);
          return;
        }

        if (!options.force) {
          warn(`This will permanently delete briefing #${id} from ${briefing.date}.`);
          info('Use --force to skip this confirmation.');
          return;
        }

        const deleted = deleteBriefing(briefingId);
        if (deleted) {
          info(`Deleted briefing #${id}.`);
        } else {
          error(`Failed to delete briefing #${id}.`);
        }
      } catch (err) {
        error(`Failed to delete briefing: ${err}`);
      }
    });

  briefings
    .command('cleanup')
    .description('Delete old briefings')
    .option('--older-than <days>', 'Delete briefings older than N days', '90')
    .option('--force', 'Skip confirmation', false)
    .action(async (options: { olderThan: string; force: boolean }) => {
      try {
        await ensureDb();
        const days = parseInt(options.olderThan, 10);

        // Get all briefings older than specified days
        const cutoffDate = new Date();
        cutoffDate.setDate(cutoffDate.getDate() - days);
        const cutoffStr = cutoffDate.toISOString().split('T')[0];

        const allBriefings = getAllBriefings(1000); // Get up to 1000
        const oldBriefings = allBriefings.filter((b) => b.date < cutoffStr);

        if (oldBriefings.length === 0) {
          info(`No briefings older than ${days} days found.`);
          return;
        }

        warn(`Found ${oldBriefings.length} briefing(s) older than ${days} days.`);

        if (!options.force) {
          info('Use --force to delete them.');
          return;
        }

        const spin = spinner(`Deleting ${oldBriefings.length} briefings...`);

        let deleted = 0;
        for (const briefing of oldBriefings) {
          if (deleteBriefing(briefing.id)) {
            deleted++;
          }
        }

        spin.succeed(`Deleted ${deleted} briefing(s).`);
      } catch (err) {
        error(`Failed to cleanup briefings: ${err}`);
      }
    });
}
