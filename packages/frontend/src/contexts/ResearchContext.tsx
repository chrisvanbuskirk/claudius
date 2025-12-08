import { createContext, useContext, useState, ReactNode } from 'react';

interface ResearchContextType {
  isResearchRunning: boolean;
  setIsResearchRunning: (running: boolean) => void;
}

const ResearchContext = createContext<ResearchContextType | undefined>(undefined);

export function ResearchProvider({ children }: { children: ReactNode }) {
  const [isResearchRunning, setIsResearchRunning] = useState(false);

  return (
    <ResearchContext.Provider value={{ isResearchRunning, setIsResearchRunning }}>
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
