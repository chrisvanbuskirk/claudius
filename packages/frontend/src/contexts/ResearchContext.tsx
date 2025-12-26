import { createContext, useContext, useState, useEffect, useRef, ReactNode } from 'react';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
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
} from '../types/research-events';

export interface ResearchProgressState {
  isRunning: boolean;
  totalTopics: number;
  currentTopicIndex: number;
  currentTopicName: string;
  currentPhase: string; // "starting", "researching", "synthesizing", "saving", "generating_images", "complete"
  topicsCompleted: {
    topicName: string;
    cardsGenerated: number;
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

  // Store unlisten functions in a ref so cleanup can access them synchronously
  const unlistenFns = useRef<UnlistenFn[]>([]);

  // Set up event listeners once at provider level
  useEffect(() => {
    let mounted = true;

    // Helper to register listener and store unlisten function
    const registerListener = async <T,>(
      eventName: string,
      handler: (event: { payload: T }) => void
    ) => {
      const unlisten = await listen<T>(eventName, (event) => {
        if (mounted) handler(event);
      });
      unlistenFns.current.push(unlisten);
    };

    // Set up all listeners
    (async () => {
      // Research started
      await registerListener<ResearchStartedEvent>('research:started', (event) => {
        console.log('[ResearchContext] research:started event:', event.payload);
        setProgress({
          isRunning: true,
          totalTopics: event.payload.total_topics,
          currentTopicIndex: -1,
          currentTopicName: '',
          currentPhase: 'starting',
          topicsCompleted: [],
          totalCards: 0,
        });
      });

      // Topic started
      await registerListener<TopicStartedEvent>('research:topic_started', (event) => {
        console.log('[ResearchContext] topic_started event:', event.payload);
        setProgress((prev) => ({
          ...prev,
          currentTopicIndex: event.payload.topic_index,
          currentTopicName: event.payload.topic_name,
          currentPhase: 'researching',
        }));
      });

      // Topic completed
      await registerListener<TopicCompletedEvent>('research:topic_completed', (event) => {
        console.log('[ResearchContext] topic_completed event:', event.payload);
        setProgress((prev) => ({
          ...prev,
          topicsCompleted: [
            ...prev.topicsCompleted,
            {
              topicName: event.payload.topic_name,
              cardsGenerated: event.payload.cards_generated,
            },
          ],
        }));
      });

      // Synthesis started
      await registerListener<SynthesisStartedEvent>('research:synthesis_started', (event) => {
        console.log('[ResearchContext] synthesis_started event:', event.payload);
        setProgress((prev) => ({
          ...prev,
          currentPhase: 'synthesizing',
          currentTopicName: 'Synthesizing research...',
        }));
      });

      // Synthesis completed
      await registerListener<SynthesisCompletedEvent>('research:synthesis_completed', (event) => {
        console.log('[ResearchContext] synthesis_completed event:', event.payload);
        setProgress((prev) => ({
          ...prev,
          totalCards: event.payload.cards_generated,
        }));
      });

      // Saving
      await registerListener<SavingEvent>('research:saving', (event) => {
        console.log('[ResearchContext] saving event:', event.payload);
        setProgress((prev) => ({
          ...prev,
          currentPhase: 'saving',
          currentTopicName: 'Saving briefing...',
        }));
      });

      // Generating images
      await registerListener<GeneratingImagesEvent>('research:generating_images', (event) => {
        console.log('[ResearchContext] generating_images event:', event.payload);
        setProgress((prev) => ({
          ...prev,
          currentPhase: 'generating_images',
          currentTopicName: `Generating images (${event.payload.total_cards} cards)...`,
        }));
      });

      // Completed
      await registerListener<CompletedEvent>('research:completed', (event) => {
        console.log('[ResearchContext] completed event:', event.payload);
        setProgress((prev) => ({
          ...prev,
          isRunning: false,
          currentPhase: 'complete',
          totalCards: event.payload.total_cards,
        }));
      });

      // Cancelled
      await registerListener<CancelledEvent>('research:cancelled', (event) => {
        console.log('[ResearchContext] cancelled event:', event.payload);
        setProgress((prev) => ({
          ...prev,
          isRunning: false,
          currentPhase: 'complete',
          error: `Research cancelled: ${event.payload.reason}`,
        }));
      });

      // Reset
      await registerListener<ResetEvent>('research:reset', () => {
        console.log('[ResearchContext] reset event');
        setProgress(initialProgressState);
      });
    })();

    // Cleanup listeners on unmount - synchronously call stored unlisten functions
    return () => {
      mounted = false;
      unlistenFns.current.forEach((unlisten) => unlisten());
      unlistenFns.current = [];
    };
  }, []);

  // Sync isResearchRunning with progress.isRunning
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
