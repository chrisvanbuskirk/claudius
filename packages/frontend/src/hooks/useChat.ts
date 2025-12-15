import { useState, useCallback, useEffect } from 'react';
import type { ChatMessage } from '../types';

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
    sendMessage,
    clearHistory,
    reloadHistory: loadHistory,
  };
}
