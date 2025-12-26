// Base event structure
export interface ResearchEvent {
  timestamp: string; // ISO 8601
}

// Event: Research session started
export interface ResearchStartedEvent extends ResearchEvent {
  total_topics: number;
  topics: string[]; // Topic names
}

// Event: MCP server connected
export interface McpConnectedEvent extends ResearchEvent {
  server_name: string;
  tool_count: number;
  tools: string[]; // Tool names
}

// Event: MCP server connection failed
export interface McpConnectionFailedEvent extends ResearchEvent {
  server_name: string;
  error: string;
}

// Event: Starting research for a specific topic
export interface TopicStartedEvent extends ResearchEvent {
  topic_name: string;
  topic_index: number; // 0-based
  total_topics: number;
}

// Event: Claude is thinking/reasoning
export interface ThinkingEvent extends ResearchEvent {
  topic_name: string;
  phase: 'initial_research' | 'tool_calling' | 'synthesis';
}

// Event: Tool execution
export interface ToolExecutedEvent extends ResearchEvent {
  topic_name: string;
  tool_name: string;
  tool_type: 'mcp' | 'brave_search' | 'builtin';
  status: 'success' | 'error';
  error?: string;
}

// Event: Topic research completed
export interface TopicCompletedEvent extends ResearchEvent {
  topic_name: string;
  topic_index: number;
  cards_generated: number;
  tools_used: number;
}

// Event: Saving results to database
export interface SavingEvent extends ResearchEvent {
  total_cards: number;
}

// Event: Generating header images (DALL-E)
export interface GeneratingImagesEvent extends ResearchEvent {
  total_cards: number;
}

// Event: Synthesis started
export interface SynthesisStartedEvent extends ResearchEvent {
  research_content_length: number;
}

// Event: Synthesis completed
export interface SynthesisCompletedEvent extends ResearchEvent {
  cards_generated: number;
  duration_ms: number;
}

// Event: Research session completed
export interface CompletedEvent extends ResearchEvent {
  total_topics: number;
  total_cards: number;
  duration_ms: number;
  success: boolean;
  error?: string;
}

// Event: Research cancelled by user
export interface CancelledEvent extends ResearchEvent {
  reason: string;
  phase: string;
  topics_completed: number;
  total_topics: number;
}

// Event: Research state reset
export interface ResetEvent extends ResearchEvent {
  reason: string;
}

// Event: Heartbeat during long operations
export interface HeartbeatEvent extends ResearchEvent {
  phase: string;
  topic_index?: number;
  message: string;
}

// Event: Web search used by Claude
export interface WebSearchEvent extends ResearchEvent {
  topic_name: string;
  search_query?: string;
  status: 'started' | 'completed';
}

// Event: Firecrawl deep extraction (Deep Research mode)
export interface DeepExtractionEvent extends ResearchEvent {
  topic_name: string;
  tool_name: string;  // e.g., "firecrawl_extract", "firecrawl_scrape", "firecrawl_search"
  target_url?: string;
  status: 'started' | 'completed';
}

// Event: Research mode error (e.g., Firecrawl mode without Firecrawl configured)
export interface ResearchModeErrorEvent extends ResearchEvent {
  mode: string;
  error: string;
}

// Union type for all events
export type ResearchProgressEvent =
  | { type: 'research:started'; data: ResearchStartedEvent }
  | { type: 'research:mcp_connected'; data: McpConnectedEvent }
  | { type: 'research:mcp_connection_failed'; data: McpConnectionFailedEvent }
  | { type: 'research:topic_started'; data: TopicStartedEvent }
  | { type: 'research:thinking'; data: ThinkingEvent }
  | { type: 'research:tool_executed'; data: ToolExecutedEvent }
  | { type: 'research:topic_completed'; data: TopicCompletedEvent }
  | { type: 'research:synthesis_started'; data: SynthesisStartedEvent }
  | { type: 'research:synthesis_completed'; data: SynthesisCompletedEvent }
  | { type: 'research:saving'; data: SavingEvent }
  | { type: 'research:generating_images'; data: GeneratingImagesEvent }
  | { type: 'research:completed'; data: CompletedEvent }
  | { type: 'research:cancelled'; data: CancelledEvent }
  | { type: 'research:reset'; data: ResetEvent }
  | { type: 'research:heartbeat'; data: HeartbeatEvent }
  | { type: 'research:web_search'; data: WebSearchEvent }
  | { type: 'research:deep_extraction'; data: DeepExtractionEvent }
  | { type: 'research:mode_error'; data: ResearchModeErrorEvent };
