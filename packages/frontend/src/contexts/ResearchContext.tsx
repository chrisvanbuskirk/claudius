import { createContext, useContext, useState, useEffect, useRef, ReactNode } from 'react';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import type {
  ResearchStartedEvent,
  TopicStartedEvent,
  TopicCompletedEvent,
  SynthesisStartedEvent,
  SynthesisCompletedEvent,
  SavingEvent,
  GeneratingImagesEvent,
  CompletedEvent,
  CancelledEvent,
  ResetEvent,
  HeartbeatEvent,
  WebSearchEvent,
  DeepExtractionEvent,
  ResearchModeErrorEvent,
} from '../types/research-events';

// Phase-specific timeouts (in milliseconds)
const PHASE_TIMEOUTS: Record<string, number> = {
  starting: 90_000,       // 90 seconds to start (MCP servers may need to initialize)
  researching: 180_000,   // 3 minutes per topic (will reset on each topic)
  synthesizing: 120_000,  // 2 minutes for synthesis
  saving: 30_000,         // 30 seconds to save
  generating_images: 300_000, // 5 minutes for image generation (DALL-E ~15-20s per image)
};
const OVERALL_TIMEOUT = 720_000;    // 12 minutes total (includes image generation)
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

const initialProgressState: ResearchProgressState = {
  isRunning: false,
  totalTopics: 0,
  currentTopicIndex: -1,
  currentTopicName: '',
  currentPhase: '',
  topicsCompleted: [],
  totalCards: 0,
};

interface ResearchContextType {
  isResearchRunning: boolean;
  setIsResearchRunning: (running: boolean) => void;
  progress: ResearchProgressState;
  setProgress: React.Dispatch<React.SetStateAction<ResearchProgressState>>;
}

const ResearchContext = createContext<ResearchContextType | undefined>(undefined);

export function ResearchProvider({ children }: { children: ReactNode }) {
  const [isResearchRunning, setIsResearchRunning] = useState(false);
  const [progress, setProgress] = useState<ResearchProgressState>(initialProgressState);

  // Refs for timeout management
  const phaseTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const overallTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const inactivityIntervalRef = useRef<NodeJS.Timeout | null>(null);
  const lastEventTimeRef = useRef<number>(Date.now());
  const currentPhaseRef = useRef<string>('');

  // Set up event listeners once at provider level
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
      console.error(`[ResearchContext] TIMEOUT: ${reason}`);
      console.log('[ResearchContext] Setting isRunning=false due to timeout');
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
        console.log('[ResearchContext] research:started event:', event.payload);
        console.log('[ResearchContext] Setting isRunning=true');
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

    // Generating images (DALL-E)
    unlistenPromises.push(
      listen<GeneratingImagesEvent>('research:generating_images', (event) => {
        if (!mounted) return;
        console.log('Generating images event:', event.payload);
        updateLastEventTime();
        startPhaseTimeout('generating_images');
        setProgress((prev) => ({
          ...prev,
          currentPhase: 'generating_images',
        }));
      })
    );

    // Completed
    unlistenPromises.push(
      listen<CompletedEvent>('research:completed', (event) => {
        if (!mounted) return;
        console.log('[ResearchContext] research:completed event:', event.payload);
        console.log('[ResearchContext] Setting isRunning=false, currentPhase=complete');

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
        setProgress(initialProgressState);
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

    // Web search - logs when Claude uses built-in web search
    unlistenPromises.push(
      listen<WebSearchEvent>('research:web_search', (event) => {
        if (!mounted) return;
        const { topic_name, search_query, status } = event.payload;
        if (status === 'started') {
          console.log(`🔍 Web search started for "${topic_name}"${search_query ? `: "${search_query}"` : ''}`);
        } else {
          console.log(`✅ Web search completed for "${topic_name}"`);
        }
        updateLastEventTime();
      })
    );

    // Deep extraction - Firecrawl deep research mode
    unlistenPromises.push(
      listen<DeepExtractionEvent>('research:deep_extraction', (event) => {
        if (!mounted) return;
        const { topic_name, tool_name, target_url, status } = event.payload;
        if (status === 'started') {
          console.log(`🔬 Deep extraction started for "${topic_name}" using ${tool_name}${target_url ? ` on ${target_url}` : ''}`);
        } else {
          console.log(`✅ Deep extraction completed for "${topic_name}" using ${tool_name}`);
        }
        updateLastEventTime();
        
        // Update phase to show deep extraction is happening
        if (status === 'started') {
          setProgress((prev) => ({
            ...prev,
            currentPhase: 'deep_extraction',
          }));
        }
      })
    );

    // Research mode error - e.g., Firecrawl mode without Firecrawl configured
    unlistenPromises.push(
      listen<ResearchModeErrorEvent>('research:mode_error', (event) => {
        if (!mounted) return;
        console.error(`Research mode error (${event.payload.mode}): ${event.payload.error}`);
        
        // Clear all timeouts since research failed
        clearAllTimeouts();
        
        setProgress((prev) => ({
          ...prev,
          isRunning: false,
          currentPhase: 'complete',
          error: event.payload.error,
        }));
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

  // Keep isResearchRunning in sync with progress.isRunning
  useEffect(() => {
    setIsResearchRunning(progress.isRunning);
  }, [progress.isRunning]);

  return (
    <ResearchContext.Provider value={{ isResearchRunning, setIsResearchRunning, progress, setProgress }}>
      {children}
    </ResearchContext.Provider>
  );
}

export function useResearch() {
  const context = useContext(ResearchContext);
  if (!context) {
    throw new Error('useResearch must be used within ResearchProvider');
  }
  return context;
}
