import { useResearch, type ResearchProgressState } from '../contexts/ResearchContext';

// Re-export the type for consumers
export type { ResearchProgressState };

/**
 * Hook to access the research progress state from context.
 * The state persists across navigation because it's managed by ResearchProvider.
 */
export function useResearchProgress(): ResearchProgressState {
  const { progress } = useResearch();
  return progress;
}
