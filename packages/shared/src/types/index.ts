/**
 * TypeScript types for Claudius database
 */

export interface Briefing {
  id: number;
  date: string;
  title: string | null;
  cards: BriefingCard[];
  research_time_ms: number | null;
  model_used: string | null;
  total_tokens: number | null;
  created_at: string;
}

export interface BriefingCard {
  title: string;
  summary: string;
  sources: Source[];
  relevance_score?: number;
}

export interface Source {
  url: string;
  title?: string;
  snippet?: string;
}

export interface BriefingCardRow {
  id: number;
  briefing_id: number;
  card_index: number | null;
  title: string | null;
  summary: string | null;
  sources: string | null; // JSON string
  relevance_score: number | null;
}

export interface Feedback {
  id: number;
  briefing_id: number;
  card_index: number | null;
  rating: number;
  reason: string | null;
  created_at: string;
}

export interface FeedbackPattern {
  id: number;
  topic: string;
  positive_count: number;
  negative_count: number;
  last_updated: string;
}

export interface ResearchLog {
  id: number;
  briefing_id: number;
  mcp_server: string | null;
  query: string | null;
  result_tokens: number | null;
  duration_ms: number | null;
  error_message: string | null;
  created_at: string;
}

export interface CreateBriefingData {
  date: string;
  title?: string;
  cards: BriefingCard[];
  research_time_ms?: number;
  model_used?: string;
  total_tokens?: number;
}

export interface AddFeedbackData {
  briefing_id: number;
  card_index?: number;
  rating: number;
  reason?: string;
}

export interface BriefingRow {
  id: number;
  date: string;
  title: string | null;
  cards: string; // JSON string
  research_time_ms: number | null;
  model_used: string | null;
  total_tokens: number | null;
  created_at: string;
}
