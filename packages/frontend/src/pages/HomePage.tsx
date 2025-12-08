import { useEffect, useState, useMemo } from 'react';
import { Link } from 'react-router-dom';
import { format } from 'date-fns';
import { RefreshCw, Loader2, AlertCircle, Calendar, Play } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { BriefingCard } from '../components/BriefingCard';
import { ActionableErrorsAlert } from '../components/ActionableErrorsAlert';
import { useBriefings } from '../hooks/useTauri';
import { useResearch } from '../contexts/ResearchContext';
import type { Briefing } from '../types';

// Backend returns briefings with cards as JSON string
interface BackendBriefing {
  id: number;
  date: string;
  title: string;
  cards: string; // JSON string of cards array
  research_time_ms?: number;
  model_used?: string;
  total_tokens?: number;
}

// Card data within the cards JSON
interface BriefingCardData {
  title: string;
  summary: string;
  sources?: string[];
  suggested_next?: string;
  relevance?: string;
  topic?: string;
}

export function HomePage() {
  const { briefings: rawBriefings, loading, error, getTodaysBriefings, submitFeedback } = useBriefings();
  const { setIsResearchRunning } = useResearch();

  // Parse the cards JSON and flatten into individual briefing cards
  const briefings = useMemo(() => {
    const result: Briefing[] = [];
    for (const raw of rawBriefings as unknown as BackendBriefing[]) {
      try {
        const cards: BriefingCardData[] = typeof raw.cards === 'string'
          ? JSON.parse(raw.cards)
          : raw.cards || [];

        for (let i = 0; i < cards.length; i++) {
          const card = cards[i];
          result.push({
            id: `${raw.id}-${i}`,
            title: card.title || raw.title,
            summary: card.summary || '',
            sources: card.sources || [],
            suggested_next: card.suggested_next,
            relevance: (card.relevance as 'high' | 'medium' | 'low') || 'medium',
            created_at: raw.date,
            topic_id: '',
            topic_name: card.topic || 'General',
          });
        }
      } catch {
        // If cards parsing fails, create a single card from the briefing
        result.push({
          id: String(raw.id),
          title: raw.title,
          summary: 'Unable to parse briefing cards',
          sources: [],
          relevance: 'medium',
          created_at: raw.date,
          topic_id: '',
          topic_name: 'General',
        });
      }
    }
    return result;
  }, [rawBriefings]);
  const [refreshing, setRefreshing] = useState(false);
  const [runningResearch, setRunningResearch] = useState(false);

  useEffect(() => {
    getTodaysBriefings();
  }, [getTodaysBriefings]);

  const handleRefresh = async () => {
    setRefreshing(true);
    try {
      await getTodaysBriefings();
    } finally {
      setRefreshing(false);
    }
  };

  const handleRunResearch = async () => {
    setRunningResearch(true);
    setIsResearchRunning(true); // Activate the purple border aura
    try {
      await invoke('run_research_now');
      await getTodaysBriefings();
    } catch (err) {
      console.error('Research failed:', err);
    } finally {
      setRunningResearch(false);
      setIsResearchRunning(false); // Deactivate the purple border aura
    }
  };

  const handleThumbsUp = (briefingId: string) => {
    submitFeedback({
      briefing_id: briefingId,
      feedback_type: 'thumbs_up',
      timestamp: new Date().toISOString(),
    });
  };

  const handleThumbsDown = (briefingId: string) => {
    submitFeedback({
      briefing_id: briefingId,
      feedback_type: 'thumbs_down',
      timestamp: new Date().toISOString(),
    });
  };

  const today = new Date();
  const formattedDate = format(today, 'EEEE, MMMM d, yyyy');

  return (
    <div>
      <div className="mb-8">
        <div className="flex items-center justify-between mb-2">
          <div>
            <h1 className="text-3xl font-bold text-gray-900 dark:text-white mb-1">
              Today's Briefings
            </h1>
            <div className="flex items-center gap-2 text-gray-600 dark:text-gray-400">
              <Calendar className="w-4 h-4" />
              <p className="text-sm">{formattedDate}</p>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <button
              onClick={handleRunResearch}
              disabled={runningResearch || loading}
              className="btn btn-primary flex items-center gap-2"
            >
              {runningResearch ? (
                <Loader2 className="w-4 h-4 animate-spin" />
              ) : (
                <Play className="w-4 h-4" />
              )}
              {runningResearch ? 'Running...' : 'Run Research'}
            </button>
            <button
              onClick={handleRefresh}
              disabled={refreshing || loading}
              className="btn btn-secondary flex items-center gap-2"
            >
              <RefreshCw className={`w-4 h-4 ${refreshing ? 'animate-spin' : ''}`} />
              Refresh
            </button>
          </div>
        </div>
      </div>

      <ActionableErrorsAlert />

      {loading && !refreshing && (
        <div className="flex items-center justify-center py-12">
          <div className="text-center">
            <Loader2 className="w-8 h-8 animate-spin text-primary-600 dark:text-primary-400 mx-auto mb-3" />
            <p className="text-gray-600 dark:text-gray-400">Loading today's briefings...</p>
          </div>
        </div>
      )}

      {error && (
        <div className="card p-6 bg-red-50 dark:bg-red-900/20 border-red-200 dark:border-red-800">
          <div className="flex items-start gap-3">
            <AlertCircle className="w-5 h-5 text-red-600 dark:text-red-400 flex-shrink-0 mt-0.5" />
            <div>
              <h3 className="font-semibold text-red-900 dark:text-red-300 mb-1">
                Error Loading Briefings
              </h3>
              <p className="text-sm text-red-700 dark:text-red-400">{error}</p>
            </div>
          </div>
        </div>
      )}

      {!loading && !error && briefings.length === 0 && (
        <div className="card p-12 text-center">
          <div className="max-w-md mx-auto">
            <div className="w-16 h-16 bg-gray-100 dark:bg-gray-800 rounded-full flex items-center justify-center mx-auto mb-4">
              <Calendar className="w-8 h-8 text-gray-400 dark:text-gray-500" />
            </div>
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">
              No Briefings Yet
            </h3>
            <p className="text-gray-600 dark:text-gray-400 mb-4">
              You don't have any briefings for today. Check your settings to configure topics and enable automatic research.
            </p>
            <Link to="/settings" className="btn btn-primary">
              Go to Settings
            </Link>
          </div>
        </div>
      )}

      <div className="space-y-6">
        {briefings.map((briefing) => (
          <BriefingCard
            key={briefing.id}
            briefing={briefing}
            onThumbsUp={() => handleThumbsUp(briefing.id)}
            onThumbsDown={() => handleThumbsDown(briefing.id)}
          />
        ))}
      </div>

      {briefings.length > 0 && (
        <div className="mt-8 text-center text-sm text-gray-500 dark:text-gray-400">
          Showing {briefings.length} briefing{briefings.length !== 1 ? 's' : ''} for today
        </div>
      )}
    </div>
  );
}
