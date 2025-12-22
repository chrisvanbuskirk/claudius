export interface Briefing {
  id: string;
  title: string;
  summary: string;
  detailed_content?: string; // Full research content (2-3 paragraphs)
  sources: string[];
  suggested_next?: string;
  relevance: 'high' | 'medium' | 'low';
  created_at: string;
  topic_id: string;
  topic_name: string;
  content?: string; // Legacy field
  metadata?: Record<string, unknown>;
  // Image generation fields (DALL-E)
  image_prompt?: string;
  image_style?: string;  // Legacy field (not used with DALL-E)
  image_path?: string;
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
  model: string;
  research_depth: 'shallow' | 'medium' | 'deep';
  max_sources_per_topic: number;
  enable_notifications: boolean;
  notification_sound: boolean;
  enable_web_search?: boolean;
  retention_days: number | null;  // null = never delete
  condense_briefings?: boolean;  // Combine all topics into one comprehensive card
  dedup_days?: number;  // Days to look back for duplicates (default: 14)
  dedup_threshold?: number;  // Similarity threshold 0-1 (default: 0.75)
  enable_image_generation?: boolean;  // Generate header images using DALL-E
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

export interface ChatMessage {
  id: number;
  briefing_id: number;
  card_index: number;
  role: 'user' | 'assistant';
  content: string;
  tokens_used?: number;
  created_at: string;
}

export interface CardWithChat {
  briefing_id: number;
  card_index: number;
}

export interface Bookmark {
  id: number;
  briefing_id: number;
  card_index: number;
  created_at: string;
}

// Backend returns briefings with cards as JSON string
export interface BackendBriefing {
  id: number;
  date: string;
  title: string;
  cards: string; // JSON string of BriefingCardData[]
  research_time_ms?: number;
  model_used?: string;
  total_tokens?: number;
}

// Card data structure within the cards JSON
export interface BriefingCardData {
  title: string;
  summary: string;
  detailed_content?: string;
  sources?: string[];
  suggested_next?: string;
  relevance?: string;
  topic?: string;
  image_prompt?: string;
  image_path?: string;
}
