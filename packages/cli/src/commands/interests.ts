import { Command } from 'commander';
import { success, error, info, table } from '../utils/output.js';
import { loadConfig, saveConfig } from '../utils/config.js';

interface Interest {
  topic: string;
  depth: number;
  addedAt: string;
}

interface InterestsConfig {
  interests: Interest[];
  blocked: string[];
}

function getInterestsConfig(): InterestsConfig {
  const config = loadConfig('interests');
  return config || { interests: [], blocked: [] };
}

function saveInterestsConfig(config: InterestsConfig): void {
  saveConfig('interests', config);
}

export function registerInterestsCommands(program: Command): void {
  const interests = program
    .command('interests')
    .description('Manage your research interests');

  interests
    .command('add <topic>')
    .description('Add a new research interest')
    .option('-d, --depth <depth>', 'Research depth (1-5)', '3')
    .action((topic: string, options: { depth: string }) => {
      try {
        const config = getInterestsConfig();
        const depth = parseInt(options.depth, 10);

        if (depth < 1 || depth > 5) {
          error('Depth must be between 1 and 5');
          return;
        }

        // Check if topic already exists
        const existing = config.interests.find(i => i.topic.toLowerCase() === topic.toLowerCase());
        if (existing) {
          error(`Interest "${topic}" already exists`);
          return;
        }

        // Check if topic is blocked
        if (config.blocked.includes(topic.toLowerCase())) {
          error(`Topic "${topic}" is blocked. Unblock it first.`);
          return;
        }

        config.interests.push({
          topic,
          depth,
          addedAt: new Date().toISOString(),
        });

        saveInterestsConfig(config);
        success(`Added interest: ${topic} (depth: ${depth})`);
      } catch (err) {
        error(`Failed to add interest: ${err}`);
      }
    });

  interests
    .command('list')
    .description('List all research interests')
    .action(() => {
      try {
        const config = getInterestsConfig();

        if (config.interests.length === 0) {
          info('No interests configured. Add some with "claudius interests add <topic>"');
          return;
        }

        const tableData = config.interests.map(i => ({
          Topic: i.topic,
          Depth: i.depth,
          'Added At': new Date(i.addedAt).toLocaleDateString(),
        }));

        table(tableData);
      } catch (err) {
        error(`Failed to list interests: ${err}`);
      }
    });

  interests
    .command('remove <topic>')
    .description('Remove a research interest')
    .action((topic: string) => {
      try {
        const config = getInterestsConfig();
        const index = config.interests.findIndex(i => i.topic.toLowerCase() === topic.toLowerCase());

        if (index === -1) {
          error(`Interest "${topic}" not found`);
          return;
        }

        config.interests.splice(index, 1);
        saveInterestsConfig(config);
        success(`Removed interest: ${topic}`);
      } catch (err) {
        error(`Failed to remove interest: ${err}`);
      }
    });

  interests
    .command('block <topic>')
    .description('Block a topic from research')
    .action((topic: string) => {
      try {
        const config = getInterestsConfig();

        if (config.blocked.includes(topic.toLowerCase())) {
          error(`Topic "${topic}" is already blocked`);
          return;
        }

        // Remove from interests if it exists
        config.interests = config.interests.filter(i => i.topic.toLowerCase() !== topic.toLowerCase());
        config.blocked.push(topic.toLowerCase());

        saveInterestsConfig(config);
        success(`Blocked topic: ${topic}`);
      } catch (err) {
        error(`Failed to block topic: ${err}`);
      }
    });

  interests
    .command('blocked')
    .description('List blocked topics')
    .action(() => {
      try {
        const config = getInterestsConfig();

        if (config.blocked.length === 0) {
          info('No blocked topics');
          return;
        }

        console.log('\nBlocked topics:');
        config.blocked.forEach(topic => {
          console.log(`  - ${topic}`);
        });
      } catch (err) {
        error(`Failed to list blocked topics: ${err}`);
      }
    });

  interests
    .command('set-depth <topic> <depth>')
    .description('Set research depth for a topic (1-5)')
    .action((topic: string, depth: string) => {
      try {
        const config = getInterestsConfig();
        const depthNum = parseInt(depth, 10);

        if (depthNum < 1 || depthNum > 5) {
          error('Depth must be between 1 and 5');
          return;
        }

        const interest = config.interests.find(i => i.topic.toLowerCase() === topic.toLowerCase());
        if (!interest) {
          error(`Interest "${topic}" not found`);
          return;
        }

        interest.depth = depthNum;
        saveInterestsConfig(config);
        success(`Set depth for "${topic}" to ${depthNum}`);
      } catch (err) {
        error(`Failed to set depth: ${err}`);
      }
    });
}
