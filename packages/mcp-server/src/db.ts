import initSqlJs, { Database } from 'sql.js';
import * as path from 'path';
import * as os from 'os';
import * as fs from 'fs';

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

export class BriefingsDB {
  private db: Database | null = null;
  private initPromise: Promise<void>;

  constructor() {
    this.initPromise = this.init();
  }

  private async init() {
    const dbPath = path.join(os.homedir(), '.claudius', 'claudius.db');
    const SQL = await initSqlJs();

    if (fs.existsSync(dbPath)) {
      const buffer = fs.readFileSync(dbPath);
      this.db = new SQL.Database(buffer);
    } else {
      this.db = new SQL.Database();
    }
  }

  private async ensureInit(): Promise<Database> {
    await this.initPromise;
    if (!this.db) {
      throw new Error('Database not initialized');
    }
    return this.db;
  }

  async getBriefings(days: number = 7, limit: number = 10) {
    const db = await this.ensureInit();
    const result = db.exec(
      `SELECT id, date, title, cards, research_time_ms, model_used
       FROM briefings
       WHERE date >= date('now', '-' || ? || ' days')
       ORDER BY date DESC
       LIMIT ?`,
      [days, limit]
    );
    return resultToRows(result);
  }

  async searchBriefings(query: string) {
    const db = await this.ensureInit();
    const pattern = `%${query}%`;
    const result = db.exec(
      `SELECT id, date, title, cards
       FROM briefings
       WHERE title LIKE ? OR cards LIKE ?
       ORDER BY date DESC
       LIMIT 20`,
      [pattern, pattern]
    );
    return resultToRows(result);
  }

  async getBriefingDetail(id: number) {
    const db = await this.ensureInit();
    const result = db.exec('SELECT * FROM briefings WHERE id = ?', [id]);
    const rows = resultToRows(result);
    return rows.length > 0 ? rows[0] : null;
  }

  async getFeedbackPatterns() {
    const db = await this.ensureInit();
    const result = db.exec(
      `SELECT topic, positive_count, negative_count
       FROM feedback_patterns
       ORDER BY (positive_count + negative_count) DESC
       LIMIT 20`
    );
    return resultToRows(result);
  }

  getInterests() {
    const configPath = path.join(os.homedir(), '.claudius', 'interests.json');
    if (fs.existsSync(configPath)) {
      return JSON.parse(fs.readFileSync(configPath, 'utf-8'));
    }
    return { topics: [] };
  }

  async getResearchStats() {
    const db = await this.ensureInit();

    const countResult = db.exec('SELECT COUNT(*) as count FROM briefings');
    const avgTimeResult = db.exec('SELECT AVG(research_time_ms) as avg_time FROM briefings');
    const tokensResult = db.exec('SELECT SUM(total_tokens) as total_tokens FROM briefings');

    const count = countResult.length > 0 ? countResult[0].values[0][0] : 0;
    const avgTime = avgTimeResult.length > 0 ? avgTimeResult[0].values[0][0] : null;
    const tokens = tokensResult.length > 0 ? tokensResult[0].values[0][0] : 0;

    return {
      total_briefings: count,
      avg_research_time_ms: avgTime,
      total_tokens_used: tokens,
    };
  }
}
