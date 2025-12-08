-- SQLite schema for Claudius

CREATE TABLE IF NOT EXISTS briefings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date TEXT NOT NULL,
    title TEXT NOT NULL,
    cards TEXT NOT NULL, -- JSON array of cards
    research_time_ms INTEGER,
    model_used TEXT,
    total_tokens INTEGER,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS feedback (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    briefing_id INTEGER NOT NULL,
    card_index INTEGER NOT NULL,
    rating INTEGER NOT NULL CHECK (rating BETWEEN 1 AND 5),
    reason TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (briefing_id) REFERENCES briefings(id) ON DELETE CASCADE
);

-- Research logs for tracking tool calls and API interactions
CREATE TABLE IF NOT EXISTS research_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    briefing_id INTEGER,              -- NULL if research failed before creating briefing
    log_type TEXT NOT NULL,           -- 'tool_call', 'api_request', 'mcp_call', 'error'
    topic TEXT,                       -- The topic being researched
    tool_name TEXT,                   -- Name of tool/MCP server called
    input_summary TEXT,               -- Brief summary of input (truncated)
    output_summary TEXT,              -- Brief summary of output (truncated)
    duration_ms INTEGER,              -- How long the operation took
    tokens_used INTEGER,              -- Tokens consumed (for API calls)
    success INTEGER NOT NULL DEFAULT 1, -- 1 = success, 0 = failure
    error_code TEXT,                  -- Error code (e.g., 'rate_limit', 'budget_exceeded')
    error_message TEXT,               -- Human-readable error message
    user_action_required INTEGER DEFAULT 0, -- 1 if user needs to take action
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (briefing_id) REFERENCES briefings(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_briefings_date ON briefings(date DESC);
CREATE INDEX IF NOT EXISTS idx_feedback_briefing ON feedback(briefing_id);
CREATE INDEX IF NOT EXISTS idx_research_logs_briefing ON research_logs(briefing_id);
CREATE INDEX IF NOT EXISTS idx_research_logs_type ON research_logs(log_type);
CREATE INDEX IF NOT EXISTS idx_research_logs_error ON research_logs(error_code) WHERE error_code IS NOT NULL;
