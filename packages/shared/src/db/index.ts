/**
 * Database initialization and management using sql.js
 */

import initSqlJs, { Database as SqlJsDatabase } from 'sql.js';
import * as fs from 'fs';
import * as path from 'path';
import { SCHEMA } from './schema.js';

let db: SqlJsDatabase | null = null;
let dbPath: string | null = null;

/**
 * Initialize the SQLite database with the schema
 * @param filePath - Path to the SQLite database file
 * @returns Promise resolving to the database instance
 */
export async function initDatabase(filePath: string): Promise<SqlJsDatabase> {
  if (db) {
    return db;
  }

  const SQL = await initSqlJs();
  dbPath = filePath;

  // Ensure directory exists
  const dir = path.dirname(filePath);
  if (!fs.existsSync(dir)) {
    fs.mkdirSync(dir, { recursive: true });
  }

  // Load existing database or create new one
  if (fs.existsSync(filePath)) {
    const buffer = fs.readFileSync(filePath);
    db = new SQL.Database(buffer);
  } else {
    db = new SQL.Database();
  }

  // Enable foreign keys
  db.run('PRAGMA foreign_keys = ON');

  // Execute schema (creates tables if they don't exist)
  db.run(SCHEMA);

  // Save to file
  saveDatabase();

  return db;
}

/**
 * Get the current database instance
 * @throws Error if database hasn't been initialized
 * @returns The database instance
 */
export function getDatabase(): SqlJsDatabase {
  if (!db) {
    throw new Error('Database not initialized. Call initDatabase() first.');
  }
  return db;
}

/**
 * Save the database to disk
 */
export function saveDatabase(): void {
  if (db && dbPath) {
    const data = db.export();
    const buffer = Buffer.from(data);
    fs.writeFileSync(dbPath, buffer);
  }
}

/**
 * Close the database connection
 */
export function closeDatabase(): void {
  if (db) {
    saveDatabase();
    db.close();
    db = null;
    dbPath = null;
  }
}

// Re-export Database type for use in other modules
export type { SqlJsDatabase as Database };
