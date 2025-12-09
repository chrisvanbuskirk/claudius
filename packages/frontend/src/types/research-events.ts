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

// Event: Research session completed
export interface CompletedEvent extends ResearchEvent {
  total_topics: number;
  total_cards: number;
  duration_ms: number;
  success: boolean;
  error?: string;
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
  | { type: 'research:saving'; data: SavingEvent }
  | { type: 'research:completed'; data: CompletedEvent };
