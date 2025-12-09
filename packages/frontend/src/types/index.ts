export interface Briefing {
  id: string;
  title: string;
  summary: string;
  sources: string[];
  suggested_next?: string;
  relevance: 'high' | 'medium' | 'low';
  created_at: string;
  topic_id: string;
  topic_name: string;
  content?: string; // Legacy field
  metadata?: Record<string, unknown>;
}

export interface Topic {
  id: string;
  name: string;
  description?: string;
  enabled: boolean;
  created_at: string;
  updated_at: string;
}

export interface MCPServer {
  id: string;
  name: string;
  enabled: boolean;
  config: Record<string, unknown>;
  last_used?: string;
}

export interface ResearchSettings {
  schedule_cron: string;
  model: string;
  research_depth: 'shallow' | 'medium' | 'deep';
  max_sources_per_topic: number;
  enable_notifications: boolean;
  notification_sound: boolean;
}

export interface UserFeedback {
  briefing_id: string;
  feedback_type: 'thumbs_up' | 'thumbs_down';
  timestamp: string;
  notes?: string;
}

export interface DailyBriefings {
  date: string;
  briefings: Briefing[];
  total_count: number;
}

export interface BriefingFilters {
  topic_id?: string;
  relevance?: 'high' | 'medium' | 'low';
  date_from?: string;
  date_to?: string;
  search_query?: string;
}
