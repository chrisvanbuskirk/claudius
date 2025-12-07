export const defaultInterests = {
  topics: [],
  keywords: {
    include: [],
    exclude: []
  },
  sources: {
    github: [],
    rss: []
  },
  research_depth: {
    default: "balanced" as const
  }
};

export const defaultMcpServers = {
  servers: []
};

export const defaultPreferences = {
  app: {
    theme: "system" as const,
    notifications: true
  },
  research: {
    schedule: "0 6 * * *",
    default_model: "claude-sonnet-4-5-20250929",
    max_tokens: 4000,
    temperature: 0.7,
    num_briefings: 4
  },
  storage: {
    db_path: "~/.claudius/claudius.db",
    briefing_retention_days: 90
  }
};
