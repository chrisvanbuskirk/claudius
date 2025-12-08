import { ReactNode } from 'react';
import { Sidebar } from './Sidebar';
import { LoadingBorderAura } from './LoadingBorderAura';
import { useResearch } from '../contexts/ResearchContext';

interface LayoutProps {
  children: ReactNode;
}

export function Layout({ children }: LayoutProps) {
  const { isResearchRunning } = useResearch();

  return (
    <div className="flex h-screen overflow-hidden bg-gray-50 dark:bg-gray-900">
      <LoadingBorderAura isActive={isResearchRunning} />
      <Sidebar />
      <main className="flex-1 overflow-y-auto">
        <div className="max-w-7xl mx-auto p-8">
          {children}
        </div>
      </main>
    </div>
  );
}
