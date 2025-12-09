import { useEffect, useState, useRef } from 'react';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import type {
  ResearchStartedEvent,
  TopicStartedEvent,
  TopicCompletedEvent,
  SynthesisStartedEvent,
  SynthesisCompletedEvent,
  SavingEvent,
  CompletedEvent,
  CancelledEvent,
  ResetEvent,
  HeartbeatEvent,
} from '../types/research-events';

// Phase-specific timeouts (in milliseconds)
const PHASE_TIMEOUTS: Record<string, number> = {
  starting: 30_000,       // 30 seconds to start
  researching: 180_000,   // 3 minutes per topic (will reset on each topic)
  synthesizing: 120_000,  // 2 minutes for synthesis
  saving: 30_000,         // 30 seconds to save
};
const OVERALL_TIMEOUT = 480_000;    // 8 minutes total
const INACTIVITY_TIMEOUT = 120_000; // 2 minutes of no events
const INACTIVITY_CHECK_INTERVAL = 30_000; // Check every 30 seconds

export interface ResearchProgressState {
  isRunning: boolean;
  totalTopics: number;
  currentTopicIndex: number;
  currentTopicName: string;
  currentPhase: string; // "starting", "researching", "synthesizing", "saving", "complete"
  topicsCompleted: {
    topicName: string;
    cardsGenerated: number;
    toolsUsed: number;
  }[];
  totalCards: number;
  error?: string;
}

const initialState: ResearchProgressState = {
  isRunning: false,
  totalTopics: 0,
  currentTopicIndex: -1,
  currentTopicName: '',
  currentPhase: '',
  topicsCompleted: [],
  totalCards: 0,
};

export function useResearchProgress() {
  const [progress, setProgress] = useState<ResearchProgressState>(initialState);

  // Refs for timeout management
  const phaseTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const overallTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const inactivityIntervalRef = useRef<NodeJS.Timeout | null>(null);
  const lastEventTimeRef = useRef<number>(Date.now());
  const currentPhaseRef = useRef<string>('');

  useEffect(() => {
    let mounted = true;
    const unlistenPromises: Promise<UnlistenFn>[] = [];

    // Helper to update last event time (resets inactivity timer)
    const updateLastEventTime = () => {
      lastEventTimeRef.current = Date.now();
    };

    // Helper to clear all timeouts
    const clearAllTimeouts = () => {
      if (phaseTimeoutRef.current) {
        clearTimeout(phaseTimeoutRef.current);
        phaseTimeoutRef.current = null;
      }
      if (overallTimeoutRef.current) {
        clearTimeout(overallTimeoutRef.current);
        overallTimeoutRef.current = null;
      }
      if (inactivityIntervalRef.current) {
        clearInterval(inactivityIntervalRef.current);
        inactivityIntervalRef.current = null;
      }
    };

    // Handler for timeouts - cancel research and show error
    const handleTimeout = async (reason: string) => {
      console.error(`Research timeout: ${reason}`);
      clearAllTimeouts();

      try {
        // Try to cancel backend research
        await invoke('cancel_research');
      } catch (e) {
        console.warn('Failed to cancel research on timeout:', e);
      }

      setProgress((prev) => ({
        ...prev,
        isRunning: false,
        currentPhase: 'complete',
        error: reason,
      }));
    };

    // Helper to start phase timeout
    const startPhaseTimeout = (phase: string) => {
      // Clear existing phase timeout
      if (phaseTimeoutRef.current) {
        clearTimeout(phaseTimeoutRef.current);
      }

      const timeout = PHASE_TIMEOUTS[phase];
      if (timeout) {
        currentPhaseRef.current = phase;
        phaseTimeoutRef.current = setTimeout(() => {
          handleTimeout(`${phase} phase timed out after ${timeout / 1000} seconds`);
        }, timeout);
        console.log(`Phase timeout set: ${phase} (${timeout / 1000}s)`);
      }
    };

    // Helper to start inactivity detection
    const startInactivityDetection = () => {
      if (inactivityIntervalRef.current) {
        clearInterval(inactivityIntervalRef.current);
      }

      inactivityIntervalRef.current = setInterval(() => {
        const elapsed = Date.now() - lastEventTimeRef.current;
        if (elapsed > INACTIVITY_TIMEOUT) {
          handleTimeout(`Research appears stuck (no activity for ${INACTIVITY_TIMEOUT / 1000} seconds)`);
        }
      }, INACTIVITY_CHECK_INTERVAL);
    };

    // Research started
    unlistenPromises.push(
      listen<ResearchStartedEvent>('research:started', (event) => {
        if (!mounted) return;
        console.log('Research started event:', event.payload);
        updateLastEventTime();

        // Clear any existing timeouts from previous session
        clearAllTimeouts();

        // Set overall timeout (8 minutes max for entire research)
        overallTimeoutRef.current = setTimeout(() => {
          handleTimeout(`Research exceeded maximum time (${OVERALL_TIMEOUT / 1000 / 60} minutes)`);
        }, OVERALL_TIMEOUT);

        // Start phase timeout for 'starting' phase
        startPhaseTimeout('starting');

        // Start inactivity detection
        startInactivityDetection();

        // Reset to fresh state for new research session
        setProgress({
          isRunning: true,
          totalTopics: event.payload.total_topics,
          currentTopicIndex: -1,
          currentTopicName: '',
          currentPhase: 'starting',
          topicsCompleted: [], // Clear any previous completions
          totalCards: 0,
        });
      })
    );

    // Topic started
    unlistenPromises.push(
      listen<TopicStartedEvent>('research:topic_started', (event) => {
        if (!mounted) return;
        console.log('Topic started event:', event.payload);
        updateLastEventTime();
        startPhaseTimeout('researching'); // Reset researching timeout for each topic
        setProgress((prev) => ({
          ...prev,
          currentTopicIndex: event.payload.topic_index,
          currentTopicName: event.payload.topic_name,
          currentPhase: 'researching',
        }));
      })
    );

    // Topic completed
    unlistenPromises.push(
      listen<TopicCompletedEvent>('research:topic_completed', (event) => {
        if (!mounted) return;
        console.log('Topic completed event:', event.payload);
        updateLastEventTime();
        setProgress((prev) => {
          // Prevent duplicate topics by checking if this topic index was already completed
          const alreadyCompleted = prev.topicsCompleted.some(
            (t) => t.topicName === event.payload.topic_name
          );

          if (alreadyCompleted) {
            console.warn('Ignoring duplicate topic_completed event for:', event.payload.topic_name);
            return prev;
          }

          return {
            ...prev,
            topicsCompleted: [
              ...prev.topicsCompleted,
              {
                topicName: event.payload.topic_name,
                cardsGenerated: event.payload.cards_generated,
                toolsUsed: event.payload.tools_used,
              },
            ],
          };
        });
      })
    );

    // Synthesis started
    unlistenPromises.push(
      listen<SynthesisStartedEvent>('research:synthesis_started', (event) => {
        if (!mounted) return;
        console.log('🧠 SYNTHESIS STARTED 🧠', event.payload);
        console.log('Phase changing to: synthesizing');
        updateLastEventTime();
        startPhaseTimeout('synthesizing');
        setProgress((prev) => ({
          ...prev,
          currentPhase: 'synthesizing',
        }));
      })
    );

    // Synthesis completed
    unlistenPromises.push(
      listen<SynthesisCompletedEvent>('research:synthesis_completed', (event) => {
        if (!mounted) return;
        console.log('✅ SYNTHESIS COMPLETED ✅', event.payload);
        console.log(`Generated ${event.payload.cards_generated} cards in ${event.payload.duration_ms}ms`);
        updateLastEventTime();
        // Keep phase as 'synthesizing' until saving starts
      })
    );

    // Saving
    unlistenPromises.push(
      listen<SavingEvent>('research:saving', (event) => {
        if (!mounted) return;
        console.log('Saving event:', event.payload);
        updateLastEventTime();
        startPhaseTimeout('saving');
        setProgress((prev) => ({
          ...prev,
          currentPhase: 'saving',
          totalCards: event.payload.total_cards,
        }));
      })
    );

    // Completed
    unlistenPromises.push(
      listen<CompletedEvent>('research:completed', (event) => {
        if (!mounted) return;
        console.log('Research completed event:', event.payload);

        // Clear all timeouts since research completed
        clearAllTimeouts();

        setProgress((prev) => ({
          ...prev,
          isRunning: false,
          currentPhase: 'complete',
          totalCards: event.payload.total_cards,
          error: event.payload.error,
        }));
      })
    );

    // Cancelled - when user cancels research
    unlistenPromises.push(
      listen<CancelledEvent>('research:cancelled', (event) => {
        if (!mounted) return;
        console.log('Research cancelled event:', event.payload);

        // Clear all timeouts
        clearAllTimeouts();

        setProgress((prev) => ({
          ...prev,
          isRunning: false,
          currentPhase: 'complete',
          error: `Research cancelled: ${event.payload.reason}`,
        }));
      })
    );

    // Reset - when state is force-reset
    unlistenPromises.push(
      listen<ResetEvent>('research:reset', (event) => {
        if (!mounted) return;
        console.log('Research reset event:', event.payload);

        // Clear all timeouts
        clearAllTimeouts();

        // Reset to initial state
        setProgress(initialState);
      })
    );

    // Heartbeat - keeps inactivity timer reset during long operations
    unlistenPromises.push(
      listen<HeartbeatEvent>('research:heartbeat', (event) => {
        if (!mounted) return;
        console.log('Research heartbeat:', event.payload.message);
        updateLastEventTime();
      })
    );

    // Cleanup all listeners on unmount
    return () => {
      mounted = false;
      clearAllTimeouts();
      Promise.all(unlistenPromises).then((unlisteners) => {
        unlisteners.forEach((unlisten) => unlisten());
      });
    };
  }, []);

  return progress;
}
