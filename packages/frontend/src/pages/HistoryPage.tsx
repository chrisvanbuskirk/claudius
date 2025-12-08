import { useEffect, useState, useMemo } from 'react';
import { Search, Filter, Calendar, Loader2, AlertCircle } from 'lucide-react';
// date-fns available for future use
import { BriefingCard } from '../components/BriefingCard';
import { useBriefings, useTopics } from '../hooks/useTauri';
import type { BriefingFilters, Briefing } from '../types';

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

export function HistoryPage() {
  const { briefings: rawBriefings, loading, error, searchBriefings, submitFeedback } = useBriefings();
  const { topics } = useTopics();
  const [showFilters, setShowFilters] = useState(false);
  const [filters, setFilters] = useState<BriefingFilters>({});
  const [searchQuery, setSearchQuery] = useState('');

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

  useEffect(() => {
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

      <div className="card p-6 mb-6">
        <div className="flex gap-3 mb-4">
          <div className="flex-1 relative">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-400" />
            <input
              type="text"
              placeholder="Search briefings..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              onKeyPress={(e) => e.key === 'Enter' && handleSearch()}
              className="input w-full pl-10"
            />
          </div>
          <button
            onClick={handleSearch}
            className="btn btn-primary"
          >
            Search
          </button>
          <button
            onClick={() => setShowFilters(!showFilters)}
            className={`btn ${activeFilterCount > 0 ? 'btn-primary' : 'btn-secondary'} flex items-center gap-2`}
          >
            <Filter className="w-4 h-4" />
            Filters
            {activeFilterCount > 0 && (
              <span className="bg-white dark:bg-gray-900 text-primary-600 dark:text-primary-400 px-2 py-0.5 rounded-full text-xs font-semibold">
                {activeFilterCount}
              </span>
            )}
          </button>
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
              <button
                onClick={handleApplyFilters}
                className="btn btn-primary"
              >
                Apply Filters
              </button>
              <button
                onClick={handleClearFilters}
                className="btn btn-secondary"
              >
                Clear All
              </button>
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

      {!loading && !error && briefings.length === 0 && (
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
          Showing {briefings.length} briefing{briefings.length !== 1 ? 's' : ''}
        </div>
      )}
    </div>
  );
}
