import { useEffect, useState, useMemo } from 'react';
import { Bookmark, Loader2, AlertCircle } from 'lucide-react';
import { motion } from 'framer-motion';
import { invoke } from '@tauri-apps/api/core';
import { BriefingCard } from '../components/BriefingCard';
import { ChatPanel } from '../components/ChatPanel';
import { useBookmarks } from '../hooks/useTauri';
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

export function BookmarksPage() {
  const { bookmarks, loading: bookmarksLoading, toggleBookmark, getBookmarks } = useBookmarks();
  const [briefingsData, setBriefingsData] = useState<Map<number, BackendBriefing>>(new Map());
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [chatOpen, setChatOpen] = useState(false);
  const [activeChatBriefing, setActiveChatBriefing] = useState<Briefing | null>(null);
  const [activeChatCardIndex, setActiveChatCardIndex] = useState<number>(0);
  const [cardsWithChats, setCardsWithChats] = useState<Set<string>>(new Set());

  // Fetch briefings data for bookmarked cards
  useEffect(() => {
    const fetchBriefingsData = async () => {
      if (bookmarks.length === 0) {
        setBriefingsData(new Map());
        setLoading(false);
        return;
      }

      setLoading(true);
      setError(null);

      try {
        // Get unique briefing IDs
        const briefingIds = [...new Set(bookmarks.map(b => b.briefing_id))];
        const data = new Map<number, BackendBriefing>();

        // Fetch each briefing
        for (const id of briefingIds) {
          try {
            const briefing = await invoke<BackendBriefing>('get_briefing', { id });
            data.set(id, briefing);
          } catch (err) {
            console.error(`Failed to fetch briefing ${id}:`, err);
          }
        }

        setBriefingsData(data);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to fetch briefings');
      } finally {
        setLoading(false);
      }
    };

    fetchBriefingsData();
  }, [bookmarks]);

  // Fetch which cards have chat history
  useEffect(() => {
    const fetchCardsWithChats = async () => {
      try {
        const cards = await invoke<CardWithChat[]>('get_cards_with_chats');
        const cardKeys = new Set(cards.map(c => `${c.briefing_id}-${c.card_index}`));
        setCardsWithChats(cardKeys);
      } catch (err) {
        console.error('Failed to fetch cards with chats:', err);
      }
    };
    fetchCardsWithChats();
  }, [chatOpen]);

  // Convert bookmarks to Briefing objects for display
  const bookmarkedBriefings = useMemo(() => {
    const result: Briefing[] = [];

    for (const bookmark of bookmarks) {
      const briefing = briefingsData.get(bookmark.briefing_id);
      if (!briefing) continue;

      try {
        const cards: BriefingCardData[] = typeof briefing.cards === 'string'
          ? JSON.parse(briefing.cards)
          : briefing.cards || [];

        const card = cards[bookmark.card_index];
        if (!card) continue;

        result.push({
          id: `${briefing.id}-${bookmark.card_index}`,
          title: card.title || briefing.title,
          summary: card.summary || '',
          detailed_content: card.detailed_content,
          sources: card.sources || [],
          suggested_next: card.suggested_next,
          relevance: (card.relevance as 'high' | 'medium' | 'low') || 'medium',
          created_at: briefing.date,
          topic_id: '',
          topic_name: card.topic || 'General',
        });
      } catch (err) {
        console.error(`Failed to parse cards for briefing ${briefing.id}:`, err);
      }
    }

    return result;
  }, [bookmarks, briefingsData]);

  const handleThumbsUp = (briefingId: string) => {
    invoke('submit_feedback', {
      feedback: {
        briefing_id: briefingId,
        feedback_type: 'thumbs_up',
        timestamp: new Date().toISOString(),
      }
    }).catch(console.error);
  };

  const handleThumbsDown = (briefingId: string) => {
    invoke('submit_feedback', {
      feedback: {
        briefing_id: briefingId,
        feedback_type: 'thumbs_down',
        timestamp: new Date().toISOString(),
      }
    }).catch(console.error);
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
      // Refresh the bookmarks list after deletion
      await getBookmarks();
    } catch (err) {
      console.error('Failed to delete briefing:', err);
    }
  };

  const handleOpenChat = (briefing: Briefing) => {
    const parts = briefing.id.split('-');
    const cardIndex = parts.length > 1 ? parseInt(parts[1], 10) : 0;
    setActiveChatBriefing(briefing);
    setActiveChatCardIndex(cardIndex);
    setChatOpen(true);
  };

  const isLoading = loading || bookmarksLoading;

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-gray-900 dark:text-white mb-2">
            Bookmarks
          </h1>
          <p className="text-gray-500 dark:text-gray-400">
            Your saved briefing cards
          </p>
        </div>
        <div className="flex items-center gap-2 text-sm text-gray-500 dark:text-gray-400">
          <Bookmark className="w-4 h-4" />
          <span>{bookmarks.length} saved</span>
        </div>
      </div>

      {/* Content */}
      {isLoading ? (
        <div className="flex items-center justify-center py-12">
          <Loader2 className="w-8 h-8 text-primary-500 animate-spin" />
        </div>
      ) : error ? (
        <div className="flex items-center gap-3 p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-xl">
          <AlertCircle className="w-5 h-5 text-red-500 flex-shrink-0" />
          <p className="text-red-700 dark:text-red-400">{error}</p>
        </div>
      ) : bookmarkedBriefings.length === 0 ? (
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          className="text-center py-16"
        >
          <div className="w-16 h-16 mx-auto mb-4 bg-gray-100 dark:bg-gray-800 rounded-full flex items-center justify-center">
            <Bookmark className="w-8 h-8 text-gray-400" />
          </div>
          <h3 className="text-xl font-semibold text-gray-900 dark:text-white mb-2">
            No bookmarks yet
          </h3>
          <p className="text-gray-500 dark:text-gray-400 max-w-md mx-auto">
            Save interesting briefing cards by clicking the bookmark icon. They'll appear here for quick access.
          </p>
        </motion.div>
      ) : (
        <div className="grid gap-6">
          {bookmarkedBriefings.map((briefing) => {
            const parts = briefing.id.split('-');
            const numericBriefingId = parseInt(parts[0], 10);
            const cardIndex = parts.length > 1 ? parseInt(parts[1], 10) : 0;
            const chatKey = `${numericBriefingId}-${cardIndex}`;

            return (
              <BriefingCard
                key={briefing.id}
                briefing={briefing}
                onThumbsUp={() => handleThumbsUp(briefing.id)}
                onThumbsDown={() => handleThumbsDown(briefing.id)}
                onOpenChat={() => handleOpenChat(briefing)}
                onBookmark={() => handleBookmark(briefing.id)}
                onDelete={() => handleDelete(briefing.id)}
                hasChat={cardsWithChats.has(chatKey)}
                isBookmarked={true}
              />
            );
          })}
        </div>
      )}

      {/* Chat Panel */}
      <ChatPanel
        briefingId={activeChatBriefing ? activeChatBriefing.id.split('-')[0] : null}
        cardIndex={activeChatCardIndex}
        briefingTitle={activeChatBriefing?.title || ''}
        isOpen={chatOpen}
        onClose={() => {
          setChatOpen(false);
          getBookmarks(); // Refresh in case something changed
        }}
      />
    </div>
  );
}
