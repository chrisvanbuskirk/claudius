import { useState } from 'react';
import { ThumbsUp, ThumbsDown, ExternalLink, ChevronDown, ChevronUp, Sparkles } from 'lucide-react';
import { formatDistanceToNow } from 'date-fns';
import type { Briefing } from '../types';

interface BriefingCardProps {
  briefing: Briefing;
  onThumbsUp: () => void;
  onThumbsDown: () => void;
}

export function BriefingCard({ briefing, onThumbsUp, onThumbsDown }: BriefingCardProps) {
  const [expanded, setExpanded] = useState(false);
  const [feedbackGiven, setFeedbackGiven] = useState<'up' | 'down' | null>(null);

  const relevanceColors = {
    high: 'bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-400 border-red-200 dark:border-red-800',
    medium: 'bg-yellow-100 dark:bg-yellow-900/30 text-yellow-700 dark:text-yellow-400 border-yellow-200 dark:border-yellow-800',
    low: 'bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-400 border-blue-200 dark:border-blue-800',
  };

  const handleThumbsUp = () => {
    if (feedbackGiven !== 'up') {
      setFeedbackGiven('up');
      onThumbsUp();
    }
  };

  const handleThumbsDown = () => {
    if (feedbackGiven !== 'down') {
      setFeedbackGiven('down');
      onThumbsDown();
    }
  };

  return (
    <div className="card p-6 hover:shadow-md transition-shadow">
      <div className="flex items-start justify-between gap-4 mb-3">
        <div className="flex-1">
          <div className="flex items-center gap-2 mb-2">
            <span className={`px-2 py-1 text-xs font-medium rounded-full border ${relevanceColors[briefing.relevance]}`}>
              {briefing.relevance.toUpperCase()}
            </span>
            <span className="text-sm text-gray-500 dark:text-gray-400">
              {briefing.topic_name}
            </span>
            <span className="text-xs text-gray-400 dark:text-gray-500">
              {formatDistanceToNow(new Date(briefing.created_at), { addSuffix: true })}
            </span>
          </div>
          <h3 className="text-xl font-semibold text-gray-900 dark:text-white mb-2">
            {briefing.title}
          </h3>
        </div>
      </div>

      <p className="text-gray-700 dark:text-gray-300 mb-4 leading-relaxed">
        {briefing.summary}
      </p>

      {expanded && briefing.content && (
        <div className="mb-4 p-4 bg-gray-50 dark:bg-gray-900/50 rounded-lg">
          <p className="text-sm text-gray-600 dark:text-gray-400 whitespace-pre-wrap">
            {briefing.content}
          </p>
        </div>
      )}

      {briefing.sources.length > 0 && (
        <div className="mb-4">
          <h4 className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            Sources ({briefing.sources.length})
          </h4>
          <ul className="space-y-1">
            {briefing.sources.slice(0, expanded ? undefined : 3).map((source, idx) => (
              <li key={idx}>
                <a
                  href={source}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-sm text-primary-600 dark:text-primary-400 hover:underline flex items-center gap-1"
                >
                  <ExternalLink className="w-3 h-3" />
                  {new URL(source).hostname}
                </a>
              </li>
            ))}
          </ul>
          {briefing.sources.length > 3 && !expanded && (
            <button
              onClick={() => setExpanded(true)}
              className="text-sm text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300 mt-1"
            >
              +{briefing.sources.length - 3} more sources
            </button>
          )}
        </div>
      )}

      {briefing.suggested_next && (
        <div className="mb-4 p-3 bg-primary-50 dark:bg-primary-900/20 rounded-lg border border-primary-200 dark:border-primary-800">
          <div className="flex items-start gap-2">
            <Sparkles className="w-4 h-4 text-primary-600 dark:text-primary-400 mt-0.5 flex-shrink-0" />
            <div>
              <h5 className="text-sm font-medium text-primary-900 dark:text-primary-300 mb-1">
                Suggested Next Step
              </h5>
              <p className="text-sm text-primary-700 dark:text-primary-400">
                {briefing.suggested_next}
              </p>
            </div>
          </div>
        </div>
      )}

      <div className="flex items-center justify-between pt-4 border-t border-gray-200 dark:border-gray-700">
        <div className="flex items-center gap-2">
          <button
            onClick={handleThumbsUp}
            className={`p-2 rounded-lg transition-colors ${
              feedbackGiven === 'up'
                ? 'bg-green-100 dark:bg-green-900/30 text-green-600 dark:text-green-400'
                : 'hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-600 dark:text-gray-400'
            }`}
            aria-label="Thumbs up"
          >
            <ThumbsUp className="w-4 h-4" />
          </button>
          <button
            onClick={handleThumbsDown}
            className={`p-2 rounded-lg transition-colors ${
              feedbackGiven === 'down'
                ? 'bg-red-100 dark:bg-red-900/30 text-red-600 dark:text-red-400'
                : 'hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-600 dark:text-gray-400'
            }`}
            aria-label="Thumbs down"
          >
            <ThumbsDown className="w-4 h-4" />
          </button>
        </div>

        {briefing.content && (
          <button
            onClick={() => setExpanded(!expanded)}
            className="flex items-center gap-1 text-sm text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white transition-colors"
          >
            {expanded ? (
              <>
                <ChevronUp className="w-4 h-4" />
                Show less
              </>
            ) : (
              <>
                <ChevronDown className="w-4 h-4" />
                Show more
              </>
            )}
          </button>
        )}
      </div>
    </div>
  );
}
