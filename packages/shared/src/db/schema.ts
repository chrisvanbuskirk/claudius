/**
 * SQLite schema for Claudius database
 */

export const SCHEMA = `
-- Briefings table
CREATE TABLE IF NOT EXISTS briefings (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  date DATE NOT NULL,
  title TEXT,
  cards JSON NOT NULL,
  research_time_ms INTEGER,
  model_used TEXT,
  total_tokens INTEGER,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Index for date-based queries
CREATE INDEX IF NOT EXISTS idx_briefings_date ON briefings(date);

-- Cards detail table
CREATE TABLE IF NOT EXISTS briefing_cards (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  briefing_id INTEGER NOT NULL,
  card_index INTEGER,
  title TEXT,
  summary TEXT,
  sources JSON,
  relevance_score FLOAT,
  FOREIGN KEY (briefing_id) REFERENCES briefings(id) ON DELETE CASCADE
);

-- Index for briefing_id lookups
CREATE INDEX IF NOT EXISTS idx_briefing_cards_briefing_id ON briefing_cards(briefing_id);

-- User feedback table
CREATE TABLE IF NOT EXISTS feedback (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  briefing_id INTEGER NOT NULL,
  card_index INTEGER,
  rating INTEGER,           -- -1 (dislike), 0 (neutral), 1 (like)
  reason TEXT,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (briefing_id) REFERENCES briefings(id) ON DELETE CASCADE
);

-- Index for feedback queries
CREATE INDEX IF NOT EXISTS idx_feedback_briefing_id ON feedback(briefing_id);
CREATE INDEX IF NOT EXISTS idx_feedback_rating ON feedback(rating);

-- Research logs table
CREATE TABLE IF NOT EXISTS research_logs (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  briefing_id INTEGER NOT NULL,
  mcp_server TEXT,
  query TEXT,
  result_tokens INTEGER,
  duration_ms INTEGER,
  error_message TEXT,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (briefing_id) REFERENCES briefings(id) ON DELETE CASCADE
);

-- Index for research log queries
CREATE INDEX IF NOT EXISTS idx_research_logs_briefing_id ON research_logs(briefing_id);

-- Feedback patterns table
CREATE TABLE IF NOT EXISTS feedback_patterns (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  topic TEXT UNIQUE NOT NULL,
  positive_count INTEGER DEFAULT 0,
  negative_count INTEGER DEFAULT 0,
  last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Index for topic lookups
CREATE INDEX IF NOT EXISTS idx_feedback_patterns_topic ON feedback_patterns(topic);
`;
