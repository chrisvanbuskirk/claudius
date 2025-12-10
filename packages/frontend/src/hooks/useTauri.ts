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

// Helper to safely invoke Tauri commands - no mock data, real config files only
async function safeInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const invoke = await getInvoke();
  if (!invoke) {
    throw new Error('Running in browser mode - Tauri not available. Run with `npm run dev:tauri` for full functionality.');
  }
  return await invoke(cmd, args) as T;
}

export function useBriefings() {
  const [briefings, setBriefings] = useState<Briefing[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const getBriefings = useCallback(async (limit?: number) => {
    setLoading(true);
    setError(null);
    try {
      const result = await safeInvoke<Briefing[]>('get_briefings', { limit });
      setBriefings(result);
      return result;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to fetch briefings';
      setError(errorMessage);
      setBriefings([]);
      return [];
    } finally {
      setLoading(false);
    }
  }, []);

  const getTodaysBriefings = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await safeInvoke<Briefing[]>('get_todays_briefings');
      setBriefings(result);
      return result;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to fetch today\'s briefings';
      setError(errorMessage);
      setBriefings([]);
      return [];
    } finally {
      setLoading(false);
    }
  }, []);

  const getBriefingById = useCallback(async (id: string) => {
    setLoading(true);
    setError(null);
    try {
      const result = await safeInvoke<Briefing>('get_briefing_by_id', { id });
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
      const query = filters.search_query || '';
      const result = await safeInvoke<Briefing[]>('search_briefings', { query });
      setBriefings(result);
      return result;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to search briefings';
      setError(errorMessage);
      setBriefings([]);
      return [];
    } finally {
      setLoading(false);
    }
  }, []);

  const submitFeedback = useCallback(async (feedback: UserFeedback) => {
    try {
      await safeInvoke('submit_feedback', { feedback });
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
      const result = await safeInvoke<Topic[]>('get_topics');
      setTopics(result);
      return result;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to fetch topics';
      setError(errorMessage);
      setTopics([]);
      return [];
    } finally {
      setLoading(false);
    }
  }, []);

  const addTopic = useCallback(async (name: string, description?: string) => {
    setLoading(true);
    setError(null);
    try {
      const result = await safeInvoke<Topic>('add_topic', { name, description });
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
      const result = await safeInvoke<Topic>('update_topic', { id, name, description, enabled });
      setTopics(prev => prev.map(t => t.id === id ? result : t));
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
      await safeInvoke('delete_topic', { id });
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
      const result = await safeInvoke<MCPServer[]>('get_mcp_servers');
      setServers(result);
      return result;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to fetch MCP servers';
      setError(errorMessage);
      setServers([]);
      return [];
    } finally {
      setLoading(false);
    }
  }, []);

  const addServer = useCallback(async (name: string, config: Record<string, unknown>) => {
    setLoading(true);
    setError(null);
    try {
      const result = await safeInvoke<MCPServer>('add_mcp_server', { name, config_data: config });
      setServers(prev => [...prev, result]);
      return result;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to add MCP server';
      setError(errorMessage);
      return null;
    } finally {
      setLoading(false);
    }
  }, []);

  const toggleServer = useCallback(async (id: string, enabled: boolean) => {
    setLoading(true);
    setError(null);
    try {
      const result = await safeInvoke<MCPServer>('toggle_mcp_server', { id, enabled });
      setServers(prev => prev.map(s => s.id === id ? result : s));
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to toggle MCP server';
      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  }, []);

  const updateServer = useCallback(async (id: string, name?: string, config?: Record<string, unknown>) => {
    setLoading(true);
    setError(null);
    try {
      const result = await safeInvoke<MCPServer>('update_mcp_server', {
        id,
        name: name || null,
        config_data: config || null
      });
      setServers(prev => prev.map(s => s.id === id ? result : s));
      return result;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to update MCP server';
      setError(errorMessage);
      return null;
    } finally {
      setLoading(false);
    }
  }, []);

  const removeServer = useCallback(async (id: string) => {
    setLoading(true);
    setError(null);
    try {
      await safeInvoke('remove_mcp_server', { id });
      setServers(prev => prev.filter(s => s.id !== id));
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to remove MCP server';
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
    addServer,
    updateServer,
    toggleServer,
    removeServer,
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
      const result = await safeInvoke<ResearchSettings>('get_settings');
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
      // Merge with current settings for full update
      const fullSettings = settings ? { ...settings, ...updates } : null;
      if (!fullSettings) {
        throw new Error('No current settings to update');
      }
      const result = await safeInvoke<ResearchSettings>('update_settings', { settings: fullSettings });
      setSettings(result);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to update settings';
      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  }, [settings]);

  const runResearch = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      await safeInvoke<string>('run_research_now');
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

// API Key Hook
export function useApiKey() {
  const [maskedKey, setMaskedKey] = useState<string | null>(null);
  const [hasKey, setHasKey] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const checkApiKey = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const masked = await safeInvoke<string | null>('get_api_key');
      const exists = await safeInvoke<boolean>('has_api_key');
      setMaskedKey(masked);
      setHasKey(exists);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to check API key';
      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  }, []);

  const setApiKey = useCallback(async (apiKey: string) => {
    setLoading(true);
    setError(null);
    try {
      await safeInvoke<void>('set_api_key', { api_key: apiKey });
      await checkApiKey();
      return true;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to set API key';
      setError(errorMessage);
      return false;
    } finally {
      setLoading(false);
    }
  }, [checkApiKey]);

  useEffect(() => {
    checkApiKey();
  }, [checkApiKey]);

  return {
    maskedKey,
    hasKey,
    loading,
    error,
    checkApiKey,
    setApiKey,
  };
}
