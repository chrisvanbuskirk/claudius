import { useEffect, useState } from 'react';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import type {
  ResearchStartedEvent,
  TopicStartedEvent,
  TopicCompletedEvent,
  SavingEvent,
  CompletedEvent,
} from '../types/research-events';

export interface ResearchProgressState {
  isRunning: boolean;
  totalTopics: number;
  currentTopicIndex: number;
  currentTopicName: string;
  currentPhase: string; // "starting", "researching", "saving", "complete"
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

  useEffect(() => {
    let mounted = true;
    let safetyTimeoutId: NodeJS.Timeout | null = null;
    const unlistenPromises: Promise<UnlistenFn>[] = [];

    // Research started
    unlistenPromises.push(
      listen<ResearchStartedEvent>('research:started', (event) => {
        if (!mounted) return;
        console.log('Research started event:', event.payload);

        // Clear any existing safety timeout
        if (safetyTimeoutId) clearTimeout(safetyTimeoutId);

        // Set 5-minute safety timeout to reset if research hangs
        safetyTimeoutId = setTimeout(() => {
          console.error('Research timeout - forcing reset after 5 minutes');
          setProgress({
            isRunning: false,
            totalTopics: 0,
            currentTopicIndex: -1,
            currentTopicName: '',
            currentPhase: 'complete',
            topicsCompleted: [],
            totalCards: 0,
            error: 'Research took too long and was stopped. Please try again.',
          });
        }, 5 * 60 * 1000);

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

    // Saving
    unlistenPromises.push(
      listen<SavingEvent>('research:saving', (event) => {
        if (!mounted) return;
        console.log('Saving event:', event.payload);
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

        // Clear safety timeout since research completed
        if (safetyTimeoutId) clearTimeout(safetyTimeoutId);

        setProgress((prev) => ({
          ...prev,
          isRunning: false,
          currentPhase: 'complete',
          totalCards: event.payload.total_cards,
          error: event.payload.error,
        }));
      })
    );

    // Cleanup all listeners on unmount
    return () => {
      mounted = false;
      if (safetyTimeoutId) clearTimeout(safetyTimeoutId);
      Promise.all(unlistenPromises).then((unlisteners) => {
        unlisteners.forEach((unlisten) => unlisten());
      });
    };
  }, []);

  return progress;
}
