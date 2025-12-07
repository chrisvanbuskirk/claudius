/**
 * Claudius Shared Package
 * Database layer, shared types, and utilities
 */

// Configuration
export * from './config/index.js';

// Database
export { initDatabase, getDatabase, closeDatabase, saveDatabase } from './db/index.js';
export { SCHEMA } from './db/schema.js';

// Briefing operations
export {
  createBriefing,
  getBriefing,
  getBriefingsByDate,
  searchBriefings,
  deleteBriefing,
  getAllBriefings,
} from './db/briefings.js';

// Feedback operations
export {
  addFeedback,
  getFeedbackForBriefing,
  getFeedbackPatterns,
  getFeedbackPattern,
  updateFeedbackPattern,
  getFeedbackStats,
  deleteFeedbackForBriefing,
} from './db/feedback.js';

// Database Types
export type {
  Briefing,
  BriefingCard,
  Source,
  BriefingCardRow,
  Feedback,
  FeedbackPattern,
  ResearchLog,
  CreateBriefingData,
  AddFeedbackData,
  BriefingRow,
} from './types/index.js';

// Configuration Types
export interface BriefingConfig {
  topics: string[];
  sources: string[];
  frequency: 'daily' | 'weekly';
  outputFormat: 'markdown' | 'html' | 'pdf';
}

export interface ResearchSource {
  id: string;
  type: 'github' | 'web' | 'arxiv' | 'rss';
  url: string;
  lastFetched?: Date;
}

export interface UserPreferences {
  apiKeys: {
    anthropic?: string;
    github?: string;
    firecrawl?: string;
  };
  defaultTopics: string[];
  outputDirectory: string;
}

// Utility functions
export function formatDate(date: Date): string {
  return date.toISOString().split('T')[0];
}

export function generateBriefingId(): string {
  return `briefing-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`;
}
