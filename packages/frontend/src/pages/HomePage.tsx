import { useEffect, useState, useMemo } from 'react';
import { Link } from 'react-router-dom';
import { format } from 'date-fns';
import { RefreshCw, Loader2, AlertCircle, Calendar, Play } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { motion } from 'framer-motion';
import { BriefingCard } from '../components/BriefingCard';
import { ChatPanel } from '../components/ChatPanel';
import { MagneticButton } from '../components/MagneticButton';
import { ActionableErrorsAlert } from '../components/ActionableErrorsAlert';
import { ResearchProgressCard } from '../components/ResearchProgressCard';
import { useBriefings, useBookmarks } from '../hooks/useTauri';
// Note: useResearch context manages isResearchRunning state internally
import { useResearchProgress } from '../hooks/useResearchProgress';
import type { Briefing, CardWithChat } from '../types';

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
  detailed_content?: string;
  sources?: string[];
  suggested_next?: string;
  relevance?: string;
  topic?: string;
}

export function HomePage() {
  const { briefings: rawBriefings, loading, error, getTodaysBriefings, /* submitFeedback */ } = useBriefings();
  const { bookmarks, toggleBookmark } = useBookmarks();
  const progress = useResearchProgress();

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
            detailed_content: card.detailed_content,
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
  const [researchError, setResearchError] = useState<string | null>(null);

  // Derive runningResearch from context progress - persists across navigation
  const runningResearch = progress.isRunning;
  const [chatOpen, setChatOpen] = useState(false);
  const [activeChatBriefing, setActiveChatBriefing] = useState<Briefing | null>(null);
  const [activeChatCardIndex, setActiveChatCardIndex] = useState<number>(0);
  const [cardsWithChats, setCardsWithChats] = useState<Set<string>>(new Set());

  useEffect(() => {
    getTodaysBriefings();
  }, [getTodaysBriefings]);

  // Refresh briefings when research completes (handles navigation during research)
  useEffect(() => {
    if (progress.currentPhase === 'complete' && !progress.isRunning) {
      getTodaysBriefings();
    }
  }, [progress.currentPhase, progress.isRunning, getTodaysBriefings]);

  // Fetch which cards have chat history
  useEffect(() => {
    const fetchCardsWithChats = async () => {
      try {
        const cards = await invoke<CardWithChat[]>('get_cards_with_chats');
        // Create a Set of "briefingId-cardIndex" strings for easy lookup
        const cardKeys = new Set(cards.map(c => `${c.briefing_id}-${c.card_index}`));
        setCardsWithChats(cardKeys);
      } catch (err) {
        console.error('Failed to fetch cards with chats:', err);
        // On error, set empty set so no dots show
        setCardsWithChats(new Set());
      }
    };
    fetchCardsWithChats();
  }, [rawBriefings, chatOpen]); // Refetch when briefings change or chat closes

  // Note: isResearchRunning is synced with progress.isRunning in ResearchContext

  const handleRefresh = async () => {
    setRefreshing(true);
    try {
      // Ensure minimum 1200ms delay so the spin animation is clearly visible
      const [_] = await Promise.all([
        getTodaysBriefings(),
        new Promise(resolve => setTimeout(resolve, 1200))
      ]);
    } finally {
      setRefreshing(false);
    }
  };

  const handleRunResearch = async () => {
    // Guard against concurrent research runs (runningResearch is derived from context)
    if (runningResearch) return;

    setResearchError(null); // Clear any previous errors

    try {
      // Add 6-minute timeout (slightly longer than backend timeout)
      const timeoutPromise = new Promise((_, reject) =>
        setTimeout(() => reject(new Error('Research timeout - please try again')), 6 * 60 * 1000)
      );

      await Promise.race([
        invoke('run_research_now'),
        timeoutPromise
      ]);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      console.error('Research failed:', errorMessage);
      setResearchError(`Research failed: ${errorMessage}`);
    } finally {
      // ALWAYS refresh briefings, even if research failed/timed out
      // This shows any cards that were successfully generated before failure
      await getTodaysBriefings();
      // Note: runningResearch state is managed by context via research events
    }
  };

  /* Thumbs handlers commented out - not currently used
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
  */

  const handleOpenChat = (briefing: Briefing) => {
    // Extract card index from the briefing id (format: "briefingId-cardIndex")
    const parts = briefing.id.split('-');
    const cardIndex = parts.length > 1 ? parseInt(parts[1], 10) : 0;
    setActiveChatBriefing(briefing);
    setActiveChatCardIndex(cardIndex);
    setChatOpen(true);
  };

  const handleCloseChat = () => {
    setChatOpen(false);
  };

  const handleBookmark = async (briefingId: string) => {
    const parts = briefingId.split('-');
    const numericBriefingId = parseInt(parts[0], 10);
    const cardIndex = parts.length > 1 ? parseInt(parts[1], 10) : 0;
    await toggleBookmark(numericBriefingId, cardIndex);
  };

  const handleDelete = async (briefingId: string) => {
    const parts = briefingId.split('-');
    const numericBriefingId = parseInt(parts[0], 10);
    try {
      await invoke('delete_briefing', { id: numericBriefingId });
      // Refresh the list after deletion
      await getTodaysBriefings();
    } catch (err) {
      console.error('Failed to delete briefing:', err);
    }
  };

  const isCardBookmarked = (briefingId: string) => {
    const parts = briefingId.split('-');
    const numericBriefingId = parseInt(parts[0], 10);
    const cardIndex = parts.length > 1 ? parseInt(parts[1], 10) : 0;
    return bookmarks.some(b => b.briefing_id === numericBriefingId && b.card_index === cardIndex);
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
            <MagneticButton
              onClick={handleRunResearch}
              disabled={runningResearch || loading}
              variant="primary"
              className="flex items-center gap-2"
            >
              {runningResearch ? (
                <Loader2 className="w-4 h-4 animate-spin" />
              ) : (
                <Play className="w-4 h-4" />
              )}
              {runningResearch ? 'Running...' : 'Run Research'}
            </MagneticButton>
            <MagneticButton
              onClick={handleRefresh}
              disabled={refreshing || loading}
              variant="secondary"
              className="flex items-center gap-2"
            >
              <RefreshCw className={`w-4 h-4 ${refreshing ? 'animate-spin' : ''}`} />
              Refresh
            </MagneticButton>
          </div>
        </div>
      </div>

      <ActionableErrorsAlert />

      <ResearchProgressCard progress={progress} />

      {researchError && (
        <div className="card p-4 bg-red-50 dark:bg-red-900/20 border-red-200 dark:border-red-800 mb-6">
          <div className="flex items-start gap-3">
            <AlertCircle className="w-5 h-5 text-red-600 dark:text-red-400 flex-shrink-0 mt-0.5" />
            <div className="flex-1">
              <h3 className="font-semibold text-red-900 dark:text-red-300 mb-1">
                Research Failed
              </h3>
              <p className="text-sm text-red-700 dark:text-red-400">{researchError}</p>
            </div>
            <button
              onClick={() => setResearchError(null)}
              className="text-red-600 dark:text-red-400 hover:text-red-800 dark:hover:text-red-200"
            >
              <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>
        </div>
      )}

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

      <motion.div
        className="space-y-6"
        variants={{
          hidden: { opacity: 0 },
          show: {
            opacity: 1,
            transition: { staggerChildren: 0.1 }
          }
        }}
        initial="hidden"
        animate="show"
      >
        {briefings.map((briefing) => (
          <motion.div
            key={briefing.id}
            variants={{
              hidden: { opacity: 0, y: 20 },
              show: { opacity: 1, y: 0 }
            }}
          >
            <BriefingCard
              briefing={briefing}
              // onThumbsUp={() => handleThumbsUp(briefing.id)}
              // onThumbsDown={() => handleThumbsDown(briefing.id)}
              onOpenChat={() => handleOpenChat(briefing)}
              onBookmark={() => handleBookmark(briefing.id)}
              onDelete={() => handleDelete(briefing.id)}
              hasChat={cardsWithChats.has(briefing.id)}
              isBookmarked={isCardBookmarked(briefing.id)}
            />
          </motion.div>
        ))}
      </motion.div>

      {briefings.length > 0 && (
        <div className="mt-8 text-center text-sm text-gray-500 dark:text-gray-400">
          Showing {briefings.length} briefing{briefings.length !== 1 ? 's' : ''} for today
        </div>
      )}

      {/* Chat Panel */}
      <ChatPanel
        briefingId={activeChatBriefing ? activeChatBriefing.id.split('-')[0] : null}
        cardIndex={activeChatCardIndex}
        briefingTitle={activeChatBriefing?.title || ''}
        isOpen={chatOpen}
        onClose={handleCloseChat}
      />
    </div>
  );
}
