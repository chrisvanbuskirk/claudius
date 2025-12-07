import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import * as dbModule from '../../src/db/index.js';
import * as fs from 'fs';
import * as path from 'path';
import os from 'os';

describe('Database Module', () => {
  let testDbPath: string;

  beforeEach(() => {
    // Ensure fresh state - close any existing db
    try {
      dbModule.closeDatabase();
    } catch {
      // Ignore - db may not be initialized
    }
    testDbPath = path.join(os.tmpdir(), `claudius-test-${Date.now()}-${Math.random()}.db`);
  });

  afterEach(() => {
    try {
      dbModule.closeDatabase();
    } catch {
      // Ignore if not initialized
    }
    if (testDbPath && fs.existsSync(testDbPath)) {
      fs.unlinkSync(testDbPath);
    }
  });

  describe('initDatabase', () => {
    it('should create a new database file', async () => {
      const db = await dbModule.initDatabase(testDbPath);

      expect(db).toBeDefined();
      expect(fs.existsSync(testDbPath)).toBe(true);
    });

    it('should create directory if it does not exist', async () => {
      const testDir = path.join(os.tmpdir(), `claudius-test-dir-${Date.now()}-${Math.random()}`);
      testDbPath = path.join(testDir, 'test.db');

      await dbModule.initDatabase(testDbPath);

      expect(fs.existsSync(testDir)).toBe(true);

      // Cleanup
      dbModule.closeDatabase();
      fs.rmSync(testDir, { recursive: true });
    });

    it('should return same instance on subsequent calls', async () => {
      const db1 = await dbModule.initDatabase(testDbPath);
      const db2 = await dbModule.initDatabase(testDbPath);

      expect(db1).toBe(db2);
    });

    it('should load existing database file', async () => {
      // Create and close a database
      await dbModule.initDatabase(testDbPath);
      dbModule.closeDatabase();

      // Re-open it
      const db = await dbModule.initDatabase(testDbPath);

      expect(db).toBeDefined();
    });
  });

  describe('getDatabase', () => {
    it('should throw error if database not initialized', () => {
      // Make sure db is closed
      try {
        dbModule.closeDatabase();
      } catch {
        // Ignore
      }

      expect(() => dbModule.getDatabase()).toThrow('Database not initialized');
    });

    it('should return database after initialization', async () => {
      await dbModule.initDatabase(testDbPath);

      const db = dbModule.getDatabase();

      expect(db).toBeDefined();
    });
  });

  describe('saveDatabase', () => {
    it('should save database to disk', async () => {
      await dbModule.initDatabase(testDbPath);

      // Execute a statement to modify the database
      const db = dbModule.getDatabase();
      db.run("INSERT INTO briefings (date, cards) VALUES ('2024-01-01', '[]')");

      dbModule.saveDatabase();

      // Verify file was written
      const stats = fs.statSync(testDbPath);
      expect(stats.size).toBeGreaterThan(0);
    });
  });

  describe('closeDatabase', () => {
    it('should close the database connection', async () => {
      await dbModule.initDatabase(testDbPath);

      dbModule.closeDatabase();

      // After closing, getDatabase should throw
      expect(() => dbModule.getDatabase()).toThrow('Database not initialized');
    });

    it('should be safe to call multiple times', async () => {
      await dbModule.initDatabase(testDbPath);

      dbModule.closeDatabase();
      dbModule.closeDatabase(); // Should not throw

      expect(() => dbModule.getDatabase()).toThrow();
    });
  });

  describe('schema', () => {
    it('should create all required tables', async () => {
      await dbModule.initDatabase(testDbPath);
      const db = dbModule.getDatabase();

      const tables = db.exec(
        "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name"
      );

      const tableNames = tables[0]?.values.map((row) => row[0]) ?? [];

      expect(tableNames).toContain('briefings');
      expect(tableNames).toContain('feedback');
      expect(tableNames).toContain('feedback_patterns');
    });
  });
});
