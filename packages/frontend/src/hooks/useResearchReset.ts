import { useCallback, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import type { CancelledEvent, ResetEvent } from '../types/research-events';

export interface ResearchResetState {
  isCancelling: boolean;
}

/**
 * Hook for cancelling and resetting research operations.
 * This is the ONE centralized method for handling research state resets.
 */
export function useResearchReset() {
  const isCancellingRef = useRef(false);

  /**
   * Cancel the currently running research operation.
   * This will signal the backend to stop and emit a cancelled event.
   */
  const cancelResearch = useCallback(async (): Promise<boolean> => {
    if (isCancellingRef.current) {
      console.warn('Cancel already in progress');
      return false;
    }

    try {
      isCancellingRef.current = true;
      console.log('Requesting research cancellation...');

      await invoke('cancel_research');

      console.log('Research cancellation requested successfully');
      return true;
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : String(error);
      console.error('Failed to cancel research:', errorMsg);

      // If the error is "No research is currently running", that's fine
      if (errorMsg.includes('No research is currently running')) {
        return true;
      }

      return false;
    } finally {
      // Reset the flag after a short delay to prevent rapid re-clicks
      setTimeout(() => {
        isCancellingRef.current = false;
      }, 500);
    }
  }, []);

  /**
   * Reset the research state completely.
   * Use this when research is stuck and needs to be force-reset.
   */
  const resetResearch = useCallback(async (): Promise<boolean> => {
    try {
      console.log('Requesting research state reset...');

      await invoke('reset_research_state');

      console.log('Research state reset successfully');
      return true;
    } catch (error) {
      console.error('Failed to reset research state:', error);
      return false;
    }
  }, []);

  /**
   * Get the current research status from the backend.
   */
  const getResearchStatus = useCallback(async () => {
    try {
      const status = await invoke<{
        is_running: boolean;
        current_phase: string;
        started_at: string | null;
        is_cancelled: boolean;
      }>('get_research_status');

      return status;
    } catch (error) {
      console.error('Failed to get research status:', error);
      return null;
    }
  }, []);

  /**
   * Cancel and reset - use when you want to force stop everything.
   */
  const cancelAndReset = useCallback(async (): Promise<boolean> => {
    console.log('Cancel and reset requested');

    // Try to cancel first
    const cancelResult = await cancelResearch();

    // Wait a moment for cancellation to propagate
    await new Promise(resolve => setTimeout(resolve, 100));

    // Then reset the state
    const resetResult = await resetResearch();

    return cancelResult || resetResult;
  }, [cancelResearch, resetResearch]);

  return {
    cancelResearch,
    resetResearch,
    getResearchStatus,
    cancelAndReset,
  };
}

/**
 * Hook to listen for research cancellation and reset events.
 * Returns callbacks for when these events occur.
 */
export function useResearchResetEvents(
  onCancelled?: (event: CancelledEvent) => void,
  onReset?: (event: ResetEvent) => void
) {
  useEffect(() => {
    const unlistenPromises: Promise<UnlistenFn>[] = [];

    // Listen for cancelled event
    unlistenPromises.push(
      listen<CancelledEvent>('research:cancelled', (event) => {
        console.log('Research cancelled event received:', event.payload);
        onCancelled?.(event.payload);
      })
    );

    // Listen for reset event
    unlistenPromises.push(
      listen<ResetEvent>('research:reset', (event) => {
        console.log('Research reset event received:', event.payload);
        onReset?.(event.payload);
      })
    );

    return () => {
      Promise.all(unlistenPromises).then((unlisteners) => {
        unlisteners.forEach((unlisten) => unlisten());
      });
    };
  }, [onCancelled, onReset]);
}
