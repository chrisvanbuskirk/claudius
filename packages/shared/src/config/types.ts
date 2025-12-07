export interface InterestsConfig {
  topics: string[];
  keywords: {
    include: string[];
    exclude: string[];
  };
  sources: {
    github: string[];
    rss: string[];
  };
  research_depth: {
    default: "surface" | "balanced" | "deep";
    [topic: string]: "surface" | "balanced" | "deep";
  };
}

export interface MCPServerConfig {
  name: string;
  type: "stdio" | "http";
  command: string;
  enabled: boolean;
  config: Record<string, unknown>;
}

export interface MCPServersConfig {
  servers: MCPServerConfig[];
}

export interface PreferencesConfig {
  app: {
    theme: "light" | "dark" | "system";
    notifications: boolean;
  };
  research: {
    schedule: string;
    default_model: string;
    max_tokens: number;
    temperature: number;
    num_briefings: number;
  };
  storage: {
    db_path: string;
    briefing_retention_days: number;
  };
}
