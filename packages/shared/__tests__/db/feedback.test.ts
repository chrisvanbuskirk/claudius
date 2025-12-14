import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import * as dbModule from '../../src/db/index.js';
import { createBriefing } from '../../src/db/briefings.js';
import {
  addFeedback,
  getFeedbackForBriefing,
  getFeedbackPatterns,
  getFeedbackPattern,
  updateFeedbackPattern,
  getFeedbackStats,
  deleteFeedbackForBriefing,
} from '../../src/db/feedback.js';
import type { CreateBriefingData, AddFeedbackData } from '../../src/types/index.js';
import * as fs from 'fs';
import * as path from 'path';
import os from 'os';

describe('Feedback Database Operations', () => {
  let testDbPath: string;
  let testBriefingId: number;

  const sampleBriefingData: CreateBriefingData = {
    date: '2024-01-15',
    title: 'Test Briefing',
    cards: [
      {
        title: 'Card 1',
        summary: 'Test summary',
        sources: [],
      },
    ],
  };

  beforeEach(async () => {
    testDbPath = path.join(os.tmpdir(), `claudius-test-${Date.now()}.db`);
    await dbModule.initDatabase(testDbPath);
    // Create a briefing to add feedback to
    const briefing = createBriefing(sampleBriefingData);
    testBriefingId = briefing.id;
  });

  afterEach(() => {
    dbModule.closeDatabase();
    try {
      if (testDbPath && fs.existsSync(testDbPath)) {
        fs.unlinkSync(testDbPath);
      }
    } catch {
      // Ignore cleanup errors - file may already be deleted
    }
  });

  describe('addFeedback', () => {
    it('should add feedback for a briefing', () => {
      const feedbackData: AddFeedbackData = {
        briefing_id: testBriefingId,
        rating: 1,
        reason: 'Great content!',
      };

      const feedback = addFeedback(feedbackData);

      expect(feedback).toBeDefined();
      expect(feedback.id).toBeGreaterThan(0);
      expect(feedback.briefing_id).toBe(testBriefingId);
      expect(feedback.rating).toBe(1);
      expect(feedback.reason).toBe('Great content!');
    });

    it('should add feedback with card_index', () => {
      const feedbackData: AddFeedbackData = {
        briefing_id: testBriefingId,
        card_index: 0,
        rating: -1,
        reason: 'Not relevant',
      };

      const feedback = addFeedback(feedbackData);

      expect(feedback.card_index).toBe(0);
      expect(feedback.rating).toBe(-1);
    });

    it('should add feedback without optional fields', () => {
      const feedbackData: AddFeedbackData = {
        briefing_id: testBriefingId,
        rating: 0,
      };

      const feedback = addFeedback(feedbackData);

      expect(feedback.card_index).toBeNull();
      expect(feedback.reason).toBeNull();
    });
  });

  describe('getFeedbackForBriefing', () => {
    it('should get all feedback for a briefing', () => {
      addFeedback({ briefing_id: testBriefingId, rating: 1 });
      addFeedback({ briefing_id: testBriefingId, rating: -1 });
      addFeedback({ briefing_id: testBriefingId, rating: 0 });

      const feedback = getFeedbackForBriefing(testBriefingId);

      expect(feedback).toHaveLength(3);
    });

    it('should return empty array for briefing with no feedback', () => {
      const feedback = getFeedbackForBriefing(testBriefingId);

      expect(feedback).toHaveLength(0);
    });

    it('should return all feedback for the briefing', () => {
      addFeedback({ briefing_id: testBriefingId, rating: 1, reason: 'First' });
      addFeedback({ briefing_id: testBriefingId, rating: -1, reason: 'Second' });

      const feedback = getFeedbackForBriefing(testBriefingId);

      expect(feedback).toHaveLength(2);
      const reasons = feedback.map((f) => f.reason);
      expect(reasons).toContain('First');
      expect(reasons).toContain('Second');
    });
  });

  describe('updateFeedbackPattern', () => {
    it('should create a new pattern for positive feedback', () => {
      const pattern = updateFeedbackPattern('Swift', true);

      expect(pattern).toBeDefined();
      expect(pattern.topic).toBe('Swift');
      expect(pattern.positive_count).toBe(1);
      expect(pattern.negative_count).toBe(0);
    });

    it('should create a new pattern for negative feedback', () => {
      const pattern = updateFeedbackPattern('React', false);

      expect(pattern.positive_count).toBe(0);
      expect(pattern.negative_count).toBe(1);
    });

    it('should update existing pattern', () => {
      updateFeedbackPattern('ML', true);
      updateFeedbackPattern('ML', true);
      const pattern = updateFeedbackPattern('ML', false);

      expect(pattern.positive_count).toBe(2);
      expect(pattern.negative_count).toBe(1);
    });
  });

  describe('getFeedbackPattern', () => {
    it('should return null for non-existent pattern', () => {
      const pattern = getFeedbackPattern('nonexistent');

      expect(pattern).toBeNull();
    });

    it('should return pattern by topic', () => {
      updateFeedbackPattern('Rust', true);

      const pattern = getFeedbackPattern('Rust');

      expect(pattern).toBeDefined();
      expect(pattern!.topic).toBe('Rust');
    });
  });

  describe('getFeedbackPatterns', () => {
    it('should return all patterns sorted by total count', () => {
      updateFeedbackPattern('Low', true);
      updateFeedbackPattern('High', true);
      updateFeedbackPattern('High', true);
      updateFeedbackPattern('High', false);
      updateFeedbackPattern('Medium', true);
      updateFeedbackPattern('Medium', false);

      const patterns = getFeedbackPatterns();

      expect(patterns).toHaveLength(3);
      // High has 3 total (2+1), Medium has 2, Low has 1
      expect(patterns[0].topic).toBe('High');
      expect(patterns[1].topic).toBe('Medium');
      expect(patterns[2].topic).toBe('Low');
    });

    it('should return empty array when no patterns', () => {
      const patterns = getFeedbackPatterns();

      expect(patterns).toHaveLength(0);
    });
  });

  describe('getFeedbackStats', () => {
    it('should return zeros when no feedback', () => {
      const stats = getFeedbackStats();

      expect(stats.total).toBe(0);
      expect(stats.positive).toBe(0);
      expect(stats.negative).toBe(0);
      expect(stats.neutral).toBe(0);
    });

    it('should aggregate feedback correctly', () => {
      addFeedback({ briefing_id: testBriefingId, rating: 1 });
      addFeedback({ briefing_id: testBriefingId, rating: 1 });
      addFeedback({ briefing_id: testBriefingId, rating: -1 });
      addFeedback({ briefing_id: testBriefingId, rating: 0 });
      addFeedback({ briefing_id: testBriefingId, rating: 0 });

      const stats = getFeedbackStats();

      expect(stats.total).toBe(5);
      expect(stats.positive).toBe(2);
      expect(stats.negative).toBe(1);
      expect(stats.neutral).toBe(2);
    });
  });

  describe('deleteFeedbackForBriefing', () => {
    it('should delete all feedback for a briefing', () => {
      addFeedback({ briefing_id: testBriefingId, rating: 1 });
      addFeedback({ briefing_id: testBriefingId, rating: -1 });

      const count = deleteFeedbackForBriefing(testBriefingId);

      expect(count).toBe(2);
      expect(getFeedbackForBriefing(testBriefingId)).toHaveLength(0);
    });

    it('should return 0 when no feedback to delete', () => {
      const count = deleteFeedbackForBriefing(testBriefingId);

      expect(count).toBe(0);
    });
  });
});
