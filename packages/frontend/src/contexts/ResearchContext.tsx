import { createContext, useContext, useState, useEffect, ReactNode } from 'react';
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

  // Set up event listeners once at provider level
  useEffect(() => {
    let mounted = true;
    const unlistenPromises: Promise<UnlistenFn>[] = [];

    // Research started
    unlistenPromises.push(
      listen<ResearchStartedEvent>('research:started', (event) => {
        if (!mounted) return;
        console.log('[ResearchContext] research:started event:', event.payload);

        // Reset to fresh state for new research session
        setProgress({
          isRunning: true,
          totalTopics: event.payload.total_topics,
          currentTopicIndex: -1,
          currentTopicName: '',
          currentPhase: 'starting',
          topicsCompleted: [],
          totalCards: 0,
        });
      })
    );

    // Topic started
    unlistenPromises.push(
      listen<TopicStartedEvent>('research:topic_started', (event) => {
        if (!mounted) return;
        console.log('[ResearchContext] topic_started event:', event.payload);
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
        console.log('[ResearchContext] topic_completed event:', event.payload);
        setProgress((prev) => ({
          ...prev,
          topicsCompleted: [
            ...prev.topicsCompleted,
            {
              topicName: event.payload.topic_name,
              cardsGenerated: event.payload.cards_generated,
              toolsUsed: event.payload.tools_used,
            },
          ],
        }));
      })
    );

    // Synthesis started
    unlistenPromises.push(
      listen<SynthesisStartedEvent>('research:synthesis_started', (event) => {
        if (!mounted) return;
        console.log('[ResearchContext] synthesis_started event:', event.payload);
        setProgress((prev) => ({
          ...prev,
          currentPhase: 'synthesizing',
          currentTopicName: 'Synthesizing research...',
        }));
      })
    );

    // Synthesis completed
    unlistenPromises.push(
      listen<SynthesisCompletedEvent>('research:synthesis_completed', (event) => {
        if (!mounted) return;
        console.log('[ResearchContext] synthesis_completed event:', event.payload);
        setProgress((prev) => ({
          ...prev,
          totalCards: event.payload.cards_generated,
        }));
      })
    );

    // Saving
    unlistenPromises.push(
      listen<SavingEvent>('research:saving', (event) => {
        if (!mounted) return;
        console.log('[ResearchContext] saving event:', event.payload);
        setProgress((prev) => ({
          ...prev,
          currentPhase: 'saving',
          currentTopicName: 'Saving briefing...',
        }));
      })
    );

    // Generating images
    unlistenPromises.push(
      listen<GeneratingImagesEvent>('research:generating_images', (event) => {
        if (!mounted) return;
        console.log('[ResearchContext] generating_images event:', event.payload);
        setProgress((prev) => ({
          ...prev,
          currentPhase: 'generating_images',
          currentTopicName: `Generating images (${event.payload.total_cards} cards)...`,
        }));
      })
    );

    // Completed
    unlistenPromises.push(
      listen<CompletedEvent>('research:completed', (event) => {
        if (!mounted) return;
        console.log('[ResearchContext] completed event:', event.payload);
        setProgress((prev) => ({
          ...prev,
          isRunning: false,
          currentPhase: 'complete',
          totalCards: event.payload.total_cards,
        }));
      })
    );

    // Cancelled
    unlistenPromises.push(
      listen<CancelledEvent>('research:cancelled', (event) => {
        if (!mounted) return;
        console.log('[ResearchContext] cancelled event:', event.payload);
        setProgress((prev) => ({
          ...prev,
          isRunning: false,
          currentPhase: 'complete',
          error: `Research cancelled: ${event.payload.reason}`,
        }));
      })
    );

    // Reset
    unlistenPromises.push(
      listen<ResetEvent>('research:reset', () => {
        if (!mounted) return;
        console.log('[ResearchContext] reset event');
        setProgress(initialProgressState);
      })
    );

    // Cleanup listeners on unmount
    return () => {
      mounted = false;
      Promise.all(unlistenPromises).then((unlisteners) => {
        unlisteners.forEach((unlisten) => unlisten());
      });
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
