import { Command } from 'commander';
import { success, error, info, spinner } from '../utils/output.js';

export function registerResearchCommands(program: Command): void {
  program
    .command('research')
    .description('Trigger research on your interests')
    .option('--now', 'Run research immediately instead of scheduling')
    .option('--topic <topic>', 'Research a specific topic (overrides interests)')
    .option('--depth <depth>', 'Research depth (1-5)', '3')
    .option('--model <model>', 'AI model to use', 'claude-opus-4')
    .option('--output <path>', 'Output path for briefing')
    .option('-v, --verbose', 'Verbose output')
    .action(async (options: {
      now?: boolean;
      topic?: string;
      depth?: string;
      model?: string;
      output?: string;
      verbose?: boolean;
    }) => {
      try {
        const depth = parseInt(options.depth || '3', 10);

        if (depth < 1 || depth > 5) {
          error('Depth must be between 1 and 5');
          return;
        }

        if (options.verbose) {
          info('Research options:');
          console.log(`  Mode: ${options.now ? 'Immediate' : 'Scheduled'}`);
          if (options.topic) console.log(`  Topic: ${options.topic}`);
          console.log(`  Depth: ${depth}`);
          console.log(`  Model: ${options.model}`);
          if (options.output) console.log(`  Output: ${options.output}`);
        }

        if (options.now) {
          const spin = spinner('Running research...');

          // Placeholder implementation
          setTimeout(() => {
            spin.succeed('Research completed');
            info('Not implemented yet: Actual research execution');
            info('This will:');
            info('  1. Load interests or use specified topic');
            info('  2. Run research using specified model');
            info('  3. Generate briefing');
            info('  4. Save to database and optional output path');
          }, 2000);
        } else {
          success('Research scheduled');
          info('Not implemented yet: Research scheduling');
          info('This will schedule research based on configured intervals');
        }
      } catch (err) {
        error(`Failed to run research: ${err}`);
      }
    });
}
