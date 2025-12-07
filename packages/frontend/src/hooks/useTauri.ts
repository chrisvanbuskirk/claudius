import { useState, useEffect, useCallback } from 'react';
import type {
  Briefing,
  Topic,
  MCPServer,
  ResearchSettings,
  UserFeedback,
  BriefingFilters,
} from '../types';

// Check if running inside Tauri - more robust check for Tauri 2.0
const isTauri = typeof window !== 'undefined' &&
  (window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__ !== undefined;

// Import invoke directly - will only work in Tauri environment
let invokeFunction: ((cmd: string, args?: Record<string, unknown>) => Promise<unknown>) | null = null;

// Initialize invoke lazily
async function getInvoke() {
  if (invokeFunction) return invokeFunction;
  if (!isTauri) return null;
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    invokeFunction = invoke;
    return invoke;
  } catch {
    console.log('Failed to load Tauri API - running in browser mode');
    return null;
  }
}

// Mock data for browser development
const mockBriefings: Briefing[] = [
  {
    id: '1',
    title: 'Swift 6 Migration Updates',
    summary: 'Swift 6 introduces strict concurrency checking by default. The migration path involves enabling warnings first, then fixing data race issues incrementally.',
    sources: ['https://swift.org/blog', 'https://developer.apple.com'],
    suggested_next: 'Review your async/await usage patterns',
    relevance: 'high',
    created_at: new Date().toISOString(),
    topic_id: 'topic-1',
    topic_name: 'Swift Development',
    content: 'Swift 6 represents a major step forward in memory safety. The new strict concurrency model helps catch data races at compile time rather than runtime. Key changes include:\n\n1. Sendable checking is now enforced by default\n2. Actor isolation is stricter\n3. New async/await patterns are recommended\n\nTo migrate, start by enabling warnings in Swift 5.10, then gradually fix issues before upgrading to Swift 6.',
  },
  {
    id: '2',
    title: 'New React 19 Features',
    summary: 'React 19 brings automatic batching improvements and new hooks for form handling. Server Components are now stable.',
    sources: ['https://react.dev/blog', 'https://github.com/facebook/react'],
    relevance: 'medium',
    created_at: new Date(Date.now() - 3600000).toISOString(),
    topic_id: 'topic-2',
    topic_name: 'React',
  },
  {
    id: '3',
    title: 'Rust 2024 Edition Preview',
    summary: 'The Rust 2024 edition brings new syntax improvements and better async support. Notable changes include gen blocks for generators.',
    sources: ['https://blog.rust-lang.org'],
    suggested_next: 'Try the nightly compiler with edition = "2024"',
    relevance: 'low',
    created_at: new Date(Date.now() - 7200000).toISOString(),
    topic_id: 'topic-3',
    topic_name: 'Rust',
  },
];

const mockTopics: Topic[] = [
  { id: '1', name: 'Swift development', description: 'iOS and macOS development with Swift', enabled: true, created_at: new Date().toISOString(), updated_at: new Date().toISOString() },
  { id: '2', name: 'Machine learning', description: 'ML and AI research', enabled: true, created_at: new Date().toISOString(), updated_at: new Date().toISOString() },
  { id: '3', name: 'React', description: 'React and frontend development', enabled: false, created_at: new Date().toISOString(), updated_at: new Date().toISOString() },
];

const mockServers: MCPServer[] = [
  { id: '1', name: 'Google Calendar', enabled: true, config: { type: 'calendar' }, last_used: new Date().toISOString() },
  { id: '2', name: 'GitHub', enabled: true, config: { type: 'github' }, last_used: new Date().toISOString() },
  { id: '3', name: 'Gmail', enabled: false, config: { type: 'email' } },
];

const mockSettings: ResearchSettings = {
  schedule_cron: '0 6 * * *',
  model: 'claude-sonnet-4-5-20250929',
  research_depth: 'medium',
  max_sources_per_topic: 10,
  enable_notifications: true,
};

// Helper to safely invoke Tauri commands with fallback
async function safeInvoke<T>(cmd: string, args?: Record<string, unknown>, mockData?: T): Promise<T> {
  const invoke = await getInvoke();
  if (!invoke) {
    console.log(`[Dev Mode] Mock response for: ${cmd}`, args);
    if (mockData !== undefined) {
      return mockData;
    }
    throw new Error('Running in browser mode - Tauri not available');
  }
  try {
    return await invoke(cmd, args) as T;
  } catch (err) {
    console.log(`[Tauri] Command ${cmd} failed, using mock data:`, err);
    if (mockData !== undefined) {
      return mockData;
    }
    throw err;
  }
}

export function useBriefings() {
  const [briefings, setBriefings] = useState<Briefing[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const getBriefings = useCallback(async (limit?: number) => {
    setLoading(true);
    setError(null);
    try {
      const result = await safeInvoke<Briefing[]>('get_briefings', { limit }, mockBriefings);
      setBriefings(result);
      return result;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to fetch briefings';
      setError(errorMessage);
      return [];
    } finally {
      setLoading(false);
    }
  }, []);

  const getTodaysBriefings = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await safeInvoke<Briefing[]>('get_todays_briefings', undefined, mockBriefings);
      setBriefings(result);
      return result;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to fetch today\'s briefings';
      setError(errorMessage);
      return [];
    } finally {
      setLoading(false);
    }
  }, []);

  const getBriefingById = useCallback(async (id: string) => {
    setLoading(true);
    setError(null);
    try {
      const result = await safeInvoke<Briefing>('get_briefing_by_id', { id }, mockBriefings[0]);
      return result;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to fetch briefing';
      setError(errorMessage);
      return null;
    } finally {
      setLoading(false);
    }
  }, []);

  const searchBriefings = useCallback(async (filters: BriefingFilters) => {
    setLoading(true);
    setError(null);
    try {
      const result = await safeInvoke<Briefing[]>('search_briefings', { filters }, mockBriefings);
      setBriefings(result);
      return result;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to search briefings';
      setError(errorMessage);
      return [];
    } finally {
      setLoading(false);
    }
  }, []);

  const submitFeedback = useCallback(async (feedback: UserFeedback) => {
    try {
      await safeInvoke('submit_feedback', { feedback }, undefined);
      console.log('Feedback submitted:', feedback);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to submit feedback';
      setError(errorMessage);
    }
  }, []);

  return {
    briefings,
    loading,
    error,
    getBriefings,
    getTodaysBriefings,
    getBriefingById,
    searchBriefings,
    submitFeedback,
  };
}

export function useTopics() {
  const [topics, setTopics] = useState<Topic[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const getTopics = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await safeInvoke<Topic[]>('get_topics', undefined, mockTopics);
      setTopics(result);
      return result;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to fetch topics';
      setError(errorMessage);
      return [];
    } finally {
      setLoading(false);
    }
  }, []);

  const addTopic = useCallback(async (name: string, description?: string) => {
    setLoading(true);
    setError(null);
    try {
      const now = new Date().toISOString();
      const newTopic: Topic = { id: Date.now().toString(), name, description, enabled: true, created_at: now, updated_at: now };
      const result = await safeInvoke<Topic>('add_topic', { name, description }, newTopic);
      setTopics(prev => [...prev, result]);
      return result;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to add topic';
      setError(errorMessage);
      return null;
    } finally {
      setLoading(false);
    }
  }, []);

  const updateTopic = useCallback(async (id: string, name?: string, description?: string, enabled?: boolean) => {
    setLoading(true);
    setError(null);
    try {
      await safeInvoke('update_topic', { id, name, description, enabled }, undefined);
      setTopics(prev => prev.map(t =>
        t.id === id
          ? { ...t, name: name ?? t.name, description: description ?? t.description, enabled: enabled ?? t.enabled }
          : t
      ));
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to update topic';
      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  }, []);

  const deleteTopic = useCallback(async (id: string) => {
    setLoading(true);
    setError(null);
    try {
      await safeInvoke('delete_topic', { id }, undefined);
      setTopics(prev => prev.filter(t => t.id !== id));
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to delete topic';
      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    getTopics();
  }, [getTopics]);

  return {
    topics,
    loading,
    error,
    getTopics,
    addTopic,
    updateTopic,
    deleteTopic,
  };
}

export function useMCPServers() {
  const [servers, setServers] = useState<MCPServer[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const getServers = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await safeInvoke<MCPServer[]>('get_mcp_servers', undefined, mockServers);
      setServers(result);
      return result;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to fetch MCP servers';
      setError(errorMessage);
      return [];
    } finally {
      setLoading(false);
    }
  }, []);

  const toggleServer = useCallback(async (id: string, enabled: boolean) => {
    setLoading(true);
    setError(null);
    try {
      await safeInvoke('toggle_mcp_server', { id, enabled }, undefined);
      setServers(prev => prev.map(s => s.id === id ? { ...s, enabled } : s));
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to toggle MCP server';
      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    getServers();
  }, [getServers]);

  return {
    servers,
    loading,
    error,
    getServers,
    toggleServer,
  };
}

export function useSettings() {
  const [settings, setSettings] = useState<ResearchSettings | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const getSettings = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await safeInvoke<ResearchSettings>('get_settings', undefined, mockSettings);
      setSettings(result);
      return result;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to fetch settings';
      setError(errorMessage);
      return null;
    } finally {
      setLoading(false);
    }
  }, []);

  const updateSettings = useCallback(async (updates: Partial<ResearchSettings>) => {
    setLoading(true);
    setError(null);
    try {
      await safeInvoke('update_settings', { settings: updates }, undefined);
      setSettings(prev => prev ? { ...prev, ...updates } : null);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to update settings';
      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  }, []);

  const runResearch = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      await safeInvoke('run_research_now', undefined, undefined);
      console.log('Research triggered');
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to run research';
      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    getSettings();
  }, [getSettings]);

  return {
    settings,
    loading,
    error,
    getSettings,
    updateSettings,
    runResearch,
  };
}
