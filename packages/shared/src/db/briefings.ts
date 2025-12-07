/**
 * Briefing database operations using sql.js
 */

import { getDatabase, saveDatabase } from './index.js';
import type {
  Briefing,
  BriefingRow,
  CreateBriefingData,
  BriefingCard,
} from '../types/index.js';

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
 * Create a new briefing
 * @param data - Briefing data
 * @returns The created briefing with ID
 */
export function createBriefing(data: CreateBriefingData): Briefing {
  const db = getDatabase();

  db.run(
    `INSERT INTO briefings (date, title, cards, research_time_ms, model_used, total_tokens)
     VALUES (?, ?, ?, ?, ?, ?)`,
    [
      data.date,
      data.title || null,
      JSON.stringify(data.cards),
      data.research_time_ms || null,
      data.model_used || null,
      data.total_tokens || null,
    ]
  );

  // Get the last inserted ID
  const idResult = db.exec('SELECT last_insert_rowid() as id');
  const briefingId = idResult[0].values[0][0] as number;

  // Insert individual cards into briefing_cards table
  data.cards.forEach((card, index) => {
    db.run(
      `INSERT INTO briefing_cards (briefing_id, card_index, title, summary, sources, relevance_score)
       VALUES (?, ?, ?, ?, ?, ?)`,
      [
        briefingId,
        index,
        card.title,
        card.summary,
        JSON.stringify(card.sources),
        card.relevance_score || null,
      ]
    );
  });

  saveDatabase();
  return getBriefing(briefingId)!;
}

/**
 * Get a single briefing by ID
 * @param id - Briefing ID
 * @returns The briefing or null if not found
 */
export function getBriefing(id: number): Briefing | null {
  const db = getDatabase();

  const result = db.exec('SELECT * FROM briefings WHERE id = ?', [id]);
  const rows = resultToRows<BriefingRow>(result);

  if (rows.length === 0) {
    return null;
  }

  const row = rows[0];
  return {
    id: row.id,
    date: row.date,
    title: row.title,
    cards: JSON.parse(row.cards) as BriefingCard[],
    research_time_ms: row.research_time_ms,
    model_used: row.model_used,
    total_tokens: row.total_tokens,
    created_at: row.created_at,
  };
}

/**
 * Get briefings within a date range
 * @param startDate - Start date (YYYY-MM-DD)
 * @param endDate - End date (YYYY-MM-DD)
 * @returns Array of briefings
 */
export function getBriefingsByDate(
  startDate: string,
  endDate: string
): Briefing[] {
  const db = getDatabase();

  const result = db.exec(
    `SELECT * FROM briefings
     WHERE date >= ? AND date <= ?
     ORDER BY date DESC, created_at DESC`,
    [startDate, endDate]
  );

  const rows = resultToRows<BriefingRow>(result);

  return rows.map((row) => ({
    id: row.id,
    date: row.date,
    title: row.title,
    cards: JSON.parse(row.cards) as BriefingCard[],
    research_time_ms: row.research_time_ms,
    model_used: row.model_used,
    total_tokens: row.total_tokens,
    created_at: row.created_at,
  }));
}

/**
 * Search briefings by text in title, cards, or summary
 * @param query - Search query
 * @returns Array of matching briefings
 */
export function searchBriefings(query: string): Briefing[] {
  const db = getDatabase();

  const searchPattern = `%${query}%`;
  const result = db.exec(
    `SELECT DISTINCT b.* FROM briefings b
     LEFT JOIN briefing_cards bc ON b.id = bc.briefing_id
     WHERE
       b.title LIKE ? OR
       b.cards LIKE ? OR
       bc.title LIKE ? OR
       bc.summary LIKE ?
     ORDER BY b.date DESC, b.created_at DESC`,
    [searchPattern, searchPattern, searchPattern, searchPattern]
  );

  const rows = resultToRows<BriefingRow>(result);

  return rows.map((row) => ({
    id: row.id,
    date: row.date,
    title: row.title,
    cards: JSON.parse(row.cards) as BriefingCard[],
    research_time_ms: row.research_time_ms,
    model_used: row.model_used,
    total_tokens: row.total_tokens,
    created_at: row.created_at,
  }));
}

/**
 * Delete a briefing by ID
 * @param id - Briefing ID
 * @returns True if deleted, false if not found
 */
export function deleteBriefing(id: number): boolean {
  const db = getDatabase();

  // Check if exists first
  const existing = getBriefing(id);
  if (!existing) return false;

  db.run('DELETE FROM briefings WHERE id = ?', [id]);
  saveDatabase();

  return true;
}

/**
 * Get all briefings
 * @param limit - Maximum number of briefings to return
 * @param offset - Number of briefings to skip
 * @returns Array of briefings
 */
export function getAllBriefings(limit = 50, offset = 0): Briefing[] {
  const db = getDatabase();

  const result = db.exec(
    `SELECT * FROM briefings
     ORDER BY date DESC, created_at DESC
     LIMIT ? OFFSET ?`,
    [limit, offset]
  );

  const rows = resultToRows<BriefingRow>(result);

  return rows.map((row) => ({
    id: row.id,
    date: row.date,
    title: row.title,
    cards: JSON.parse(row.cards) as BriefingCard[],
    research_time_ms: row.research_time_ms,
    model_used: row.model_used,
    total_tokens: row.total_tokens,
    created_at: row.created_at,
  }));
}
