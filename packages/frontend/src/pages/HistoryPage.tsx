import { useEffect, useState, useMemo, useRef } from 'react';
import { Search, Filter, Calendar, Loader2, AlertCircle } from 'lucide-react';
import { motion } from 'framer-motion';
import { invoke } from '@tauri-apps/api/core';
// date-fns available for future use
import { BriefingCard } from '../components/BriefingCard';
import { ChatPanel } from '../components/ChatPanel';
import { MagneticButton } from '../components/MagneticButton';
import { useBriefings, useTopics, useBookmarks } from '../hooks/useTauri';
import type { BriefingFilters, Briefing, CardWithChat, BackendBriefing, BriefingCardData } from '../types';

export function HistoryPage() {
  const { briefings: rawBriefings, loading, error, searchBriefings, /* submitFeedback */ } = useBriefings();
  const { topics } = useTopics();
  const { bookmarks, toggleBookmark } = useBookmarks();
  const [showFilters, setShowFilters] = useState(false);
  const [filters, setFilters] = useState<BriefingFilters>({});
  const [searchQuery, setSearchQuery] = useState('');
  const initialLoadDone = useRef(false);
  const [chatOpen, setChatOpen] = useState(false);
  const [activeChatBriefing, setActiveChatBriefing] = useState<Briefing | null>(null);
  const [activeChatCardIndex, setActiveChatCardIndex] = useState<number>(0);
  const [cardsWithChats, setCardsWithChats] = useState<Set<string>>(new Set());

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
      }
    };
    fetchCardsWithChats();
  }, [rawBriefings, chatOpen]); // Refetch when briefings change or chat closes

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
            image_prompt: card.image_prompt,
            image_path: card.image_path,
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

  // Apply client-side filters (relevance, topic, date, AND search query at card level)
  const filteredBriefings = useMemo(() => {
    const result = briefings.filter(briefing => {
      // Filter by search query at card level (backend searches briefings, we filter cards)
      if (searchQuery) {
        const query = searchQuery.toLowerCase();
        const matchesTitle = briefing.title.toLowerCase().includes(query);
        const matchesSummary = briefing.summary.toLowerCase().includes(query);
        const matchesTopic = briefing.topic_name?.toLowerCase().includes(query);
        if (!matchesTitle && !matchesSummary && !matchesTopic) {
          return false;
        }
      }
      // Filter by relevance
      if (filters.relevance && briefing.relevance !== filters.relevance) {
        return false;
      }
      // Filter by topic (match by topic name since we don't have topic_id)
      if (filters.topic_id) {
        const matchingTopic = topics.find(t => t.id === filters.topic_id);
        if (matchingTopic && briefing.topic_name.toLowerCase() !== matchingTopic.name.toLowerCase()) {
          return false;
        }
      }
      // Filter by date
      if (filters.date_from) {
        const briefingDate = new Date(briefing.created_at);
        const filterDate = new Date(filters.date_from);
        if (briefingDate < filterDate) {
          return false;
        }
      }
      return true;
    });
    return result;
  }, [briefings, filters, topics, searchQuery]);

  useEffect(() => {
    // Only run initial load once, even if effect re-runs (React Strict Mode, HMR, etc.)
    if (initialLoadDone.current) return;
    initialLoadDone.current = true;
    searchBriefings({});
  }, [searchBriefings]);

  const handleSearch = () => {
    searchBriefings({
      ...filters,
      search_query: searchQuery || undefined,
    });
  };

  const handleFilterChange = (key: keyof BriefingFilters, value: string | undefined) => {
    const newFilters = { ...filters };
    if (value) {
      newFilters[key] = value as never;
    } else {
      delete newFilters[key];
    }
    setFilters(newFilters);
  };

  const handleApplyFilters = () => {
    searchBriefings({
      ...filters,
      search_query: searchQuery || undefined,
    });
  };

  const handleClearFilters = () => {
    setFilters({});
    setSearchQuery('');
    searchBriefings({});
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
      await searchBriefings(filters);
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

  const activeFilterCount = Object.keys(filters).length + (searchQuery ? 1 : 0);

  return (
    <div>
      <div className="mb-8">
        <h1 className="text-3xl font-bold text-gray-900 dark:text-white mb-1">
          Briefing History
        </h1>
        <p className="text-gray-600 dark:text-gray-400">
          Search and browse your past briefings
        </p>
      </div>

      <div className="glass-card p-6 mb-6">
        <div className="flex gap-3 mb-4">
          <div className="flex-1 relative">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-400" />
            <input
              type="text"
              placeholder="Search briefings..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleSearch()}
              className="input w-full pl-10"
            />
          </div>
          <MagneticButton
            onClick={handleSearch}
            variant="primary"
            className="flex items-center gap-2"
          >
            <Search className="w-4 h-4" />
            Search
          </MagneticButton>
          <MagneticButton
            onClick={() => setShowFilters(!showFilters)}
            variant={activeFilterCount > 0 ? 'primary' : 'secondary'}
            className="flex items-center gap-2"
          >
            <Filter className="w-4 h-4" />
            Filters
            {activeFilterCount > 0 && (
              <span className="bg-white dark:bg-gray-900 text-primary-600 dark:text-primary-400 px-2 py-0.5 rounded-full text-xs font-semibold">
                {activeFilterCount}
              </span>
            )}
          </MagneticButton>
        </div>

        {showFilters && (
          <div className="pt-4 border-t border-gray-200 dark:border-gray-700">
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Topic
                </label>
                <select
                  value={filters.topic_id || ''}
                  onChange={(e) => handleFilterChange('topic_id', e.target.value || undefined)}
                  className="input w-full"
                >
                  <option value="">All Topics</option>
                  {topics.map((topic) => (
                    <option key={topic.id} value={topic.id}>
                      {topic.name}
                    </option>
                  ))}
                </select>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Relevance
                </label>
                <select
                  value={filters.relevance || ''}
                  onChange={(e) => handleFilterChange('relevance', e.target.value || undefined)}
                  className="input w-full"
                >
                  <option value="">All Levels</option>
                  <option value="high">High</option>
                  <option value="medium">Medium</option>
                  <option value="low">Low</option>
                </select>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Date From
                </label>
                <input
                  type="date"
                  value={filters.date_from || ''}
                  onChange={(e) => handleFilterChange('date_from', e.target.value || undefined)}
                  className="input w-full"
                />
              </div>
            </div>

            <div className="flex gap-3">
              <MagneticButton
                onClick={handleApplyFilters}
                variant="primary"
              >
                Apply Filters
              </MagneticButton>
              <MagneticButton
                onClick={handleClearFilters}
                variant="secondary"
              >
                Clear All
              </MagneticButton>
            </div>
          </div>
        )}
      </div>

      {loading && (
        <div className="flex items-center justify-center py-12">
          <div className="text-center">
            <Loader2 className="w-8 h-8 animate-spin text-primary-600 dark:text-primary-400 mx-auto mb-3" />
            <p className="text-gray-600 dark:text-gray-400">Loading briefings...</p>
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

      {!loading && !error && filteredBriefings.length === 0 && (
        <div className="card p-12 text-center">
          <div className="max-w-md mx-auto">
            <div className="w-16 h-16 bg-gray-100 dark:bg-gray-800 rounded-full flex items-center justify-center mx-auto mb-4">
              <Calendar className="w-8 h-8 text-gray-400 dark:text-gray-500" />
            </div>
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">
              No Briefings Found
            </h3>
            <p className="text-gray-600 dark:text-gray-400">
              {activeFilterCount > 0
                ? 'Try adjusting your filters to see more results.'
                : 'You don\'t have any briefings yet. Check back after your first research run.'}
            </p>
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
        {filteredBriefings.map((briefing) => (
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

      {filteredBriefings.length > 0 && (
        <div className="mt-8 text-center text-sm text-gray-500 dark:text-gray-400">
          Showing {filteredBriefings.length} briefing{filteredBriefings.length !== 1 ? 's' : ''}
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
