/**
 * Feedback database operations using sql.js
 */

import { getDatabase, saveDatabase } from './index.js';
import type { Feedback, FeedbackPattern, AddFeedbackData } from '../types/index.js';

/**
 * Helper to convert sql.js result to typed rows
 */
function resultToRows<T>(result: any[]): T[] {
  if (!result || result.length === 0) return [];
  const { columns, values } = result[0];
  return values.map((row: any[]) => {
    const obj: any = {};
    columns.forEach((col: string, i: number) => {
      obj[col] = row[i];
    });
    return obj as T;
  });
}

/**
 * Add feedback for a briefing or specific card
 * @param data - Feedback data
 * @returns The created feedback with ID
 */
export function addFeedback(data: AddFeedbackData): Feedback {
  const db = getDatabase();

  db.run(
    `INSERT INTO feedback (briefing_id, card_index, rating, reason)
     VALUES (?, ?, ?, ?)`,
    [
      data.briefing_id,
      data.card_index ?? null,
      data.rating,
      data.reason ?? null,
    ]
  );

  const idResult = db.exec('SELECT last_insert_rowid() as id');
  const feedbackId = idResult[0].values[0][0] as number;

  const result = db.exec('SELECT * FROM feedback WHERE id = ?', [feedbackId]);
  const rows = resultToRows<Feedback>(result);

  saveDatabase();
  return rows[0];
}

/**
 * Get all feedback for a specific briefing
 * @param briefingId - Briefing ID
 * @returns Array of feedback entries
 */
export function getFeedbackForBriefing(briefingId: number): Feedback[] {
  const db = getDatabase();

  const result = db.exec(
    `SELECT * FROM feedback
     WHERE briefing_id = ?
     ORDER BY created_at DESC`,
    [briefingId]
  );

  return resultToRows<Feedback>(result);
}

/**
 * Get all feedback patterns
 * @returns Array of feedback patterns
 */
export function getFeedbackPatterns(): FeedbackPattern[] {
  const db = getDatabase();

  const result = db.exec(
    `SELECT * FROM feedback_patterns
     ORDER BY (positive_count + negative_count) DESC, topic ASC`
  );

  return resultToRows<FeedbackPattern>(result);
}

/**
 * Get a specific feedback pattern by topic
 * @param topic - Topic name
 * @returns The feedback pattern or null if not found
 */
export function getFeedbackPattern(topic: string): FeedbackPattern | null {
  const db = getDatabase();

  const result = db.exec(
    'SELECT * FROM feedback_patterns WHERE topic = ?',
    [topic]
  );
  const rows = resultToRows<FeedbackPattern>(result);

  return rows.length > 0 ? rows[0] : null;
}

/**
 * Update or create a feedback pattern
 * @param topic - Topic name
 * @param isPositive - Whether the feedback is positive
 * @returns The updated feedback pattern
 */
export function updateFeedbackPattern(
  topic: string,
  isPositive: boolean
): FeedbackPattern {
  const db = getDatabase();

  const existing = getFeedbackPattern(topic);

  if (existing) {
    // Update existing pattern
    db.run(
      `UPDATE feedback_patterns
       SET
         positive_count = positive_count + ?,
         negative_count = negative_count + ?,
         last_updated = CURRENT_TIMESTAMP
       WHERE topic = ?`,
      [isPositive ? 1 : 0, isPositive ? 0 : 1, topic]
    );
  } else {
    // Create new pattern
    db.run(
      `INSERT INTO feedback_patterns (topic, positive_count, negative_count)
       VALUES (?, ?, ?)`,
      [topic, isPositive ? 1 : 0, isPositive ? 0 : 1]
    );
  }

  saveDatabase();
  return getFeedbackPattern(topic)!;
}

/**
 * Get feedback statistics
 * @returns Object with overall feedback statistics
 */
export function getFeedbackStats(): {
  total: number;
  positive: number;
  negative: number;
  neutral: number;
} {
  const db = getDatabase();

  const result = db.exec(
    `SELECT
       COUNT(*) as total,
       SUM(CASE WHEN rating > 0 THEN 1 ELSE 0 END) as positive,
       SUM(CASE WHEN rating < 0 THEN 1 ELSE 0 END) as negative,
       SUM(CASE WHEN rating = 0 THEN 1 ELSE 0 END) as neutral
     FROM feedback`
  );

  if (!result || result.length === 0) {
    return { total: 0, positive: 0, negative: 0, neutral: 0 };
  }

  const rows = resultToRows<{
    total: number;
    positive: number | null;
    negative: number | null;
    neutral: number | null;
  }>(result);

  const row = rows[0];
  return {
    total: row.total || 0,
    positive: row.positive || 0,
    negative: row.negative || 0,
    neutral: row.neutral || 0,
  };
}

/**
 * Delete all feedback for a briefing
 * @param briefingId - Briefing ID
 * @returns Number of feedback entries deleted
 */
export function deleteFeedbackForBriefing(briefingId: number): number {
  const db = getDatabase();

  // Get count before deletion
  const countResult = db.exec(
    'SELECT COUNT(*) as count FROM feedback WHERE briefing_id = ?',
    [briefingId]
  );
  const count = countResult.length > 0 ? countResult[0].values[0][0] as number : 0;

  db.run('DELETE FROM feedback WHERE briefing_id = ?', [briefingId]);
  saveDatabase();

  return count;
}
