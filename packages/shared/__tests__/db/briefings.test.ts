import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import initSqlJs from 'sql.js';
import * as dbModule from '../../src/db/index.js';
import {
  createBriefing,
  getBriefing,
  getBriefingsByDate,
  searchBriefings,
  deleteBriefing,
  getAllBriefings,
} from '../../src/db/briefings.js';
import { SCHEMA } from '../../src/db/schema.js';
import type { CreateBriefingData } from '../../src/types/index.js';
import * as fs from 'fs';
import * as path from 'path';
import os from 'os';

describe('Briefings Database Operations', () => {
  let testDbPath: string;

  beforeEach(async () => {
    // Create a temp db path for each test
    testDbPath = path.join(os.tmpdir(), `claudius-test-${Date.now()}.db`);
    await dbModule.initDatabase(testDbPath);
  });

  afterEach(() => {
    dbModule.closeDatabase();
    // Clean up temp file
    if (fs.existsSync(testDbPath)) {
      fs.unlinkSync(testDbPath);
    }
  });

  const sampleBriefingData: CreateBriefingData = {
    date: '2024-01-15',
    title: 'Test Briefing',
    cards: [
      {
        title: 'Card 1',
        summary: 'This is a test card summary',
        sources: [{ url: 'https://example.com', title: 'Example' }],
        relevance_score: 0.9,
      },
      {
        title: 'Card 2',
        summary: 'Another test card',
        sources: [{ url: 'https://test.com' }],
      },
    ],
    research_time_ms: 5000,
    model_used: 'claude-3-sonnet',
    total_tokens: 1500,
  };

  describe('createBriefing', () => {
    it('should create a briefing and return it with an ID', () => {
      const briefing = createBriefing(sampleBriefingData);

      expect(briefing).toBeDefined();
      expect(briefing.id).toBeGreaterThan(0);
      expect(briefing.date).toBe('2024-01-15');
      expect(briefing.title).toBe('Test Briefing');
      expect(briefing.cards).toHaveLength(2);
      expect(briefing.cards[0].title).toBe('Card 1');
      expect(briefing.research_time_ms).toBe(5000);
      expect(briefing.model_used).toBe('claude-3-sonnet');
    });

    it('should create a briefing without optional fields', () => {
      const minimalData: CreateBriefingData = {
        date: '2024-01-16',
        cards: [
          {
            title: 'Minimal Card',
            summary: 'Minimal summary',
            sources: [],
          },
        ],
      };

      const briefing = createBriefing(minimalData);

      expect(briefing.id).toBeGreaterThan(0);
      expect(briefing.title).toBeNull();
      expect(briefing.research_time_ms).toBeNull();
      expect(briefing.model_used).toBeNull();
    });

    it('should create multiple briefings with unique IDs', () => {
      const briefing1 = createBriefing(sampleBriefingData);
      const briefing2 = createBriefing({
        ...sampleBriefingData,
        title: 'Second Briefing',
      });

      expect(briefing1.id).not.toBe(briefing2.id);
    });
  });

  describe('getBriefing', () => {
    it('should retrieve a briefing by ID', () => {
      const created = createBriefing(sampleBriefingData);
      const retrieved = getBriefing(created.id);

      expect(retrieved).toBeDefined();
      expect(retrieved!.id).toBe(created.id);
      expect(retrieved!.title).toBe('Test Briefing');
      expect(retrieved!.cards).toHaveLength(2);
    });

    it('should return null for non-existent briefing', () => {
      const retrieved = getBriefing(99999);

      expect(retrieved).toBeNull();
    });
  });

  describe('getBriefingsByDate', () => {
    it('should retrieve briefings within date range', () => {
      createBriefing({ ...sampleBriefingData, date: '2024-01-10' });
      createBriefing({ ...sampleBriefingData, date: '2024-01-15' });
      createBriefing({ ...sampleBriefingData, date: '2024-01-20' });

      const briefings = getBriefingsByDate('2024-01-12', '2024-01-18');

      expect(briefings).toHaveLength(1);
      expect(briefings[0].date).toBe('2024-01-15');
    });

    it('should return empty array when no briefings in range', () => {
      createBriefing(sampleBriefingData);

      const briefings = getBriefingsByDate('2025-01-01', '2025-01-31');

      expect(briefings).toHaveLength(0);
    });

    it('should order by date descending', () => {
      createBriefing({ ...sampleBriefingData, date: '2024-01-10' });
      createBriefing({ ...sampleBriefingData, date: '2024-01-20' });
      createBriefing({ ...sampleBriefingData, date: '2024-01-15' });

      const briefings = getBriefingsByDate('2024-01-01', '2024-01-31');

      expect(briefings[0].date).toBe('2024-01-20');
      expect(briefings[1].date).toBe('2024-01-15');
      expect(briefings[2].date).toBe('2024-01-10');
    });
  });

  describe('searchBriefings', () => {
    it('should find briefings by title', () => {
      createBriefing({ ...sampleBriefingData, title: 'Swift Updates' });
      createBriefing({ ...sampleBriefingData, title: 'React News' });

      const results = searchBriefings('Swift');

      expect(results).toHaveLength(1);
      expect(results[0].title).toBe('Swift Updates');
    });

    it('should find briefings by card content', () => {
      createBriefing({
        ...sampleBriefingData,
        cards: [
          {
            title: 'Machine Learning',
            summary: 'New advances in ML',
            sources: [],
          },
        ],
      });

      const results = searchBriefings('Machine');

      expect(results).toHaveLength(1);
    });

    it('should return empty array when no matches', () => {
      createBriefing(sampleBriefingData);

      const results = searchBriefings('nonexistent');

      expect(results).toHaveLength(0);
    });

    it('should be case-insensitive', () => {
      createBriefing({ ...sampleBriefingData, title: 'UPPERCASE TITLE' });

      const results = searchBriefings('uppercase');

      expect(results).toHaveLength(1);
    });
  });

  describe('deleteBriefing', () => {
    it('should delete an existing briefing', () => {
      const briefing = createBriefing(sampleBriefingData);
      const result = deleteBriefing(briefing.id);

      expect(result).toBe(true);
      expect(getBriefing(briefing.id)).toBeNull();
    });

    it('should return false for non-existent briefing', () => {
      const result = deleteBriefing(99999);

      expect(result).toBe(false);
    });
  });

  describe('getAllBriefings', () => {
    it('should return all briefings with default limit', () => {
      for (let i = 0; i < 5; i++) {
        createBriefing({ ...sampleBriefingData, title: `Briefing ${i}` });
      }

      const briefings = getAllBriefings();

      expect(briefings).toHaveLength(5);
    });

    it('should respect limit parameter', () => {
      for (let i = 0; i < 10; i++) {
        createBriefing({ ...sampleBriefingData, title: `Briefing ${i}` });
      }

      const briefings = getAllBriefings(3);

      expect(briefings).toHaveLength(3);
    });

    it('should respect offset parameter', () => {
      for (let i = 0; i < 5; i++) {
        createBriefing({ ...sampleBriefingData, date: `2024-01-${10 + i}` });
      }

      const briefings = getAllBriefings(2, 2);

      expect(briefings).toHaveLength(2);
    });

    it('should order by date descending', () => {
      createBriefing({ ...sampleBriefingData, date: '2024-01-10' });
      createBriefing({ ...sampleBriefingData, date: '2024-01-20' });

      const briefings = getAllBriefings();

      expect(briefings[0].date).toBe('2024-01-20');
      expect(briefings[1].date).toBe('2024-01-10');
    });
  });
});
