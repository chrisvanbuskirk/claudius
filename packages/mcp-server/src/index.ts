#!/usr/bin/env node
import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
} from '@modelcontextprotocol/sdk/types.js';
import { BriefingsDB } from './db.js';

const db = new BriefingsDB();

const server = new Server(
  { name: 'claudius', version: '0.2.0' },
  { capabilities: { tools: {} } }
);

// List available tools
server.setRequestHandler(ListToolsRequestSchema, async () => ({
  tools: [
    {
      name: 'get_briefings',
      description: 'Get recent briefings from Claudius',
      inputSchema: {
        type: 'object',
        properties: {
          days: { type: 'number', description: 'Number of days to look back (default: 7)' },
          limit: { type: 'number', description: 'Maximum briefings to return (default: 10)' }
        }
      }
    },
    {
      name: 'search_briefings',
      description: 'Search briefings by keyword',
      inputSchema: {
        type: 'object',
        properties: {
          query: { type: 'string', description: 'Search query' }
        },
        required: ['query']
      }
    },
    {
      name: 'get_briefing_detail',
      description: 'Get full details of a specific briefing',
      inputSchema: {
        type: 'object',
        properties: {
          briefing_id: { type: 'number', description: 'Briefing ID' }
        },
        required: ['briefing_id']
      }
    },
    {
      name: 'get_feedback_patterns',
      description: 'Get user feedback patterns (liked/disliked topics)',
      inputSchema: { type: 'object', properties: {} }
    },
    {
      name: 'get_interests',
      description: 'Get currently configured research interests',
      inputSchema: { type: 'object', properties: {} }
    },
    {
      name: 'get_research_stats',
      description: 'Get research statistics',
      inputSchema: { type: 'object', properties: {} }
    }
  ]
}));

// Handle tool calls
server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const { name, arguments: args } = request.params;
  const typedArgs = args as Record<string, unknown> | undefined;

  switch (name) {
    case 'get_briefings': {
      const days = typedArgs?.days as number | undefined;
      const limit = typedArgs?.limit as number | undefined;
      return { content: [{ type: 'text', text: JSON.stringify(await db.getBriefings(days, limit)) }] };
    }
    case 'search_briefings': {
      const query = typedArgs?.query as string;
      return { content: [{ type: 'text', text: JSON.stringify(await db.searchBriefings(query)) }] };
    }
    case 'get_briefing_detail': {
      const briefingId = typedArgs?.briefing_id as number;
      return { content: [{ type: 'text', text: JSON.stringify(await db.getBriefingDetail(briefingId)) }] };
    }
    case 'get_feedback_patterns':
      return { content: [{ type: 'text', text: JSON.stringify(await db.getFeedbackPatterns()) }] };
    case 'get_interests':
      return { content: [{ type: 'text', text: JSON.stringify(db.getInterests()) }] };
    case 'get_research_stats':
      return { content: [{ type: 'text', text: JSON.stringify(await db.getResearchStats()) }] };
    default:
      throw new Error(`Unknown tool: ${name}`);
  }
});

async function main() {
  const transport = new StdioServerTransport();
  await server.connect(transport);
}

main().catch(console.error);
