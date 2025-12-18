import { useState, useCallback, useEffect, useRef } from 'react';
import type { ChatMessage } from '../types';

// Types for tool activity events
interface ChatToolStartEvent {
  tool_name: string;
  briefing_id: number;
  card_index: number;
}

interface ChatToolCompleteEvent {
  tool_name: string;
  success: boolean;
  briefing_id: number;
  card_index: number;
}

// Check if running inside Tauri
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

// Helper to safely invoke Tauri commands
async function safeInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const invoke = await getInvoke();
  if (!invoke) {
    throw new Error('Running in browser mode - Tauri not available.');
  }
  return await invoke(cmd, args) as T;
}

export function useChat(briefingId: string | null, cardIndex: number) {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [loading, setLoading] = useState(false);
  const [sending, setSending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [toolActivity, setToolActivity] = useState<string | null>(null);
  const unlistenersRef = useRef<(() => void)[]>([]);

  // Set up event listeners for tool activity
  useEffect(() => {
    if (!isTauri) return;

    const setupListeners = async () => {
      try {
        const { listen } = await import('@tauri-apps/api/event');

        // Clean up any existing listeners
        unlistenersRef.current.forEach(unlisten => unlisten());
        unlistenersRef.current = [];

        // Listen for tool start events
        const unlistenStart = await listen<ChatToolStartEvent>('chat:tool_start', (event) => {
          console.log('[Chat] Tool start event:', event.payload);
          if (briefingId && event.payload.briefing_id === parseInt(briefingId, 10) &&
              event.payload.card_index === cardIndex) {
            console.log('[Chat] Setting tool activity:', event.payload.tool_name);
            setToolActivity(`Using ${event.payload.tool_name}...`);
          }
        });
        unlistenersRef.current.push(unlistenStart);

        // Listen for tool complete events
        const unlistenComplete = await listen<ChatToolCompleteEvent>('chat:tool_complete', (event) => {
          console.log('[Chat] Tool complete event:', event.payload);
          if (briefingId && event.payload.briefing_id === parseInt(briefingId, 10) &&
              event.payload.card_index === cardIndex) {
            // Clear tool activity after a delay to show completion
            setTimeout(() => {
              console.log('[Chat] Clearing tool activity');
              setToolActivity(null);
            }, 500);
          }
        });
        unlistenersRef.current.push(unlistenComplete);
      } catch (err) {
        console.log('Failed to set up event listeners:', err);
      }
    };

    setupListeners();

    return () => {
      unlistenersRef.current.forEach(unlisten => unlisten());
      unlistenersRef.current = [];
    };
  }, [briefingId, cardIndex]);

  // Load chat history when briefingId or cardIndex changes
  const loadHistory = useCallback(async () => {
    if (!briefingId) {
      setMessages([]);
      return;
    }

    setLoading(true);
    setError(null);
    try {
      const result = await safeInvoke<ChatMessage[]>('get_chat_history', {
        briefingId: parseInt(briefingId, 10),
        cardIndex,
      });
      setMessages(result);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to load chat history';
      setError(errorMessage);
      setMessages([]);
    } finally {
      setLoading(false);
    }
  }, [briefingId, cardIndex]);

  // Send a message and get response
  const sendMessage = useCallback(async (message: string): Promise<ChatMessage | null> => {
    if (!briefingId || !message.trim()) {
      return null;
    }

    setSending(true);
    setError(null);

    // Optimistically add user message to UI
    const tempUserMessage: ChatMessage = {
      id: Date.now(), // Temporary ID
      briefing_id: parseInt(briefingId, 10),
      card_index: cardIndex,
      role: 'user',
      content: message.trim(),
      created_at: new Date().toISOString(),
    };
    setMessages(prev => [...prev, tempUserMessage]);

    try {
      const response = await safeInvoke<ChatMessage>('send_chat_message', {
        briefingId: parseInt(briefingId, 10),
        cardIndex,
        message: message.trim(),
      });

      // Update messages with actual data (user message was saved server-side)
      // The response is the assistant's message
      setMessages(prev => {
        // Replace temp user message with one that has real ID, then add assistant message
        // For simplicity, just reload history to get correct IDs
        return [...prev.slice(0, -1), tempUserMessage, response];
      });

      return response;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to send message';
      setError(errorMessage);
      // Remove the optimistic user message on error
      setMessages(prev => prev.slice(0, -1));
      return null;
    } finally {
      setSending(false);
    }
  }, [briefingId, cardIndex]);

  // Clear chat history
  const clearHistory = useCallback(async () => {
    if (!briefingId) return;

    try {
      await safeInvoke<number>('clear_chat_history', {
        briefingId: parseInt(briefingId, 10),
        cardIndex,
      });
      setMessages([]);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to clear chat history';
      setError(errorMessage);
    }
  }, [briefingId, cardIndex]);

  // Load history when briefingId or cardIndex changes
  useEffect(() => {
    loadHistory();
  }, [loadHistory]);

  return {
    messages,
    loading,
    sending,
    error,
    toolActivity,
    sendMessage,
    clearHistory,
    reloadHistory: loadHistory,
  };
}
