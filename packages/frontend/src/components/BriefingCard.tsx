import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { /* ThumbsUp, ThumbsDown, */ ExternalLink, ChevronDown, ChevronUp, Sparkles, MessageCircle, Bookmark, X, AlertTriangle } from 'lucide-react';
import { formatDistanceToNow, parseISO } from 'date-fns';
import { convertFileSrc } from '@tauri-apps/api/core';
import type { Briefing } from '../types';

// Delete Confirmation Dialog
function DeleteConfirmDialog({
  isOpen,
  isBookmarked,
  onConfirm,
  onCancel,
}: {
  isOpen: boolean;
  isBookmarked: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}) {
  if (!isOpen) return null;

  return (
    <AnimatePresence>
      <motion.div
        className="fixed inset-0 z-50 flex items-center justify-center"
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
      >
        <div
          className="absolute inset-0 bg-black/50 backdrop-blur-sm"
          onClick={onCancel}
        />
        <motion.div
          className="relative z-10 bg-white dark:bg-gray-800 rounded-xl shadow-2xl p-6 max-w-md w-full mx-4 border border-gray-200 dark:border-gray-700"
          initial={{ scale: 0.95, opacity: 0 }}
          animate={{ scale: 1, opacity: 1 }}
          exit={{ scale: 0.95, opacity: 0 }}
        >
          <div className="flex items-start gap-4">
            <div className={`p-2 rounded-full ${isBookmarked ? 'bg-amber-100 dark:bg-amber-900/30' : 'bg-red-100 dark:bg-red-900/30'}`}>
              <AlertTriangle className={`w-6 h-6 ${isBookmarked ? 'text-amber-600 dark:text-amber-400' : 'text-red-600 dark:text-red-400'}`} />
            </div>
            <div className="flex-1">
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">
                Delete Briefing Card
              </h3>
              {isBookmarked ? (
                <p className="text-sm text-gray-600 dark:text-gray-400 mb-4">
                  <span className="text-amber-600 dark:text-amber-400 font-medium">This card is bookmarked.</span> Are you sure you want to delete it? This action cannot be undone.
                </p>
              ) : (
                <p className="text-sm text-gray-600 dark:text-gray-400 mb-4">
                  Are you sure you want to delete this card? This will also delete any chat history associated with it.
                </p>
              )}
              <div className="flex gap-3 justify-end">
                <button
                  onClick={onCancel}
                  className="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 rounded-lg transition-colors"
                >
                  Cancel
                </button>
                <button
                  onClick={onConfirm}
                  className="px-4 py-2 text-sm font-medium text-white bg-red-600 hover:bg-red-700 rounded-lg transition-colors"
                >
                  Delete
                </button>
              </div>
            </div>
          </div>
        </motion.div>
      </motion.div>
    </AnimatePresence>
  );
}

// Parse date string as local time (not UTC)
// "2025-12-08" should be today in local time, not yesterday
function parseLocalDate(dateStr: string): Date {
  // If it's a date-only string (YYYY-MM-DD), append local midnight time
  if (/^\d{4}-\d{2}-\d{2}$/.test(dateStr)) {
    return parseISO(dateStr + 'T12:00:00'); // Use noon to avoid DST edge cases
  }
  return new Date(dateStr);
}

// Check if a string is a valid URL
function isValidUrl(str: string): boolean {
  try {
    new URL(str);
    return true;
  } catch {
    return false;
  }
}

// Generate a placeholder gradient based on a string hash
function generatePlaceholderGradient(str: string): string {
  const hash = str.split('').reduce((acc, char) => acc + char.charCodeAt(0), 0);
  const hue1 = hash % 360;
  const hue2 = (hash * 7) % 360;
  const angle = (hash * 13) % 180;
  return `linear-gradient(${angle}deg, hsl(${hue1}, 60%, 45%), hsl(${hue2}, 50%, 35%))`;
}

// Get display text for a source (hostname if URL, otherwise the string itself)
function getSourceDisplay(source: string): { href: string | null; text: string } {
  if (isValidUrl(source)) {
    try {
      return { href: source, text: new URL(source).hostname };
    } catch {
      return { href: null, text: source };
    }
  }
  return { href: null, text: source };
}

interface BriefingCardProps {
  briefing: Briefing;
  // Thumbs up/down commented out - not currently used for anything
  // onThumbsUp: () => void;
  // onThumbsDown: () => void;
  onOpenChat: () => void;
  onBookmark: () => void;
  onDelete?: () => void;
  hasChat?: boolean;
  isBookmarked?: boolean;
}

export function BriefingCard({ briefing, /* onThumbsUp, onThumbsDown, */ onOpenChat, onBookmark, onDelete, hasChat, isBookmarked }: BriefingCardProps) {
  const [expanded, setExpanded] = useState(false);
  // const [feedbackGiven, setFeedbackGiven] = useState<'up' | 'down' | null>(null);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);

  const handleDelete = () => {
    setShowDeleteConfirm(true);
  };

  const confirmDelete = () => {
    setShowDeleteConfirm(false);
    onDelete?.();
  };

  // Default values for optional fields
  const relevance = (briefing.relevance || 'medium') as 'high' | 'medium' | 'low';
  const sources = briefing.sources || [];
  const topicName = briefing.topic_name || 'General';

  const relevanceColors = {
    high: 'bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-400 border-red-200 dark:border-red-800',
    medium: 'bg-yellow-100 dark:bg-yellow-900/30 text-yellow-700 dark:text-yellow-400 border-yellow-200 dark:border-yellow-800',
    low: 'bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-400 border-blue-200 dark:border-blue-800',
  };

  /* Thumbs handlers commented out - not currently used
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
  */

  return (
    <>
    <motion.div
      className="glass-card p-6 relative"
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      whileHover={{
        scale: 1.01,
        boxShadow: "0 0 30px rgba(139, 92, 246, 0.4), 0 8px 32px 0 rgba(0, 0, 0, 0.37)"
      }}
      transition={{ duration: 0.2 }}
    >
      {/* Delete button - always visible but subtle, more prominent on hover */}
      {onDelete && (
        <button
          onClick={handleDelete}
          className="absolute top-3 right-3 p-1.5 rounded-lg bg-gray-500/20 hover:bg-red-500/30 text-gray-400 hover:text-red-400 transition-colors z-10"
          aria-label="Delete card"
        >
          <X className="w-4 h-4" />
        </button>
      )}

      {/* Header image or placeholder gradient */}
      <div className="relative -mx-6 -mt-6 mb-4 h-80 overflow-hidden rounded-t-xl">
        {briefing.image_path ? (
          <img
            src={convertFileSrc(briefing.image_path)}
            alt=""
            className="w-full h-full object-cover"
            onError={(e) => {
              // Fall back to gradient on error
              const target = e.target as HTMLImageElement;
              target.style.display = 'none';
              target.parentElement!.style.background = generatePlaceholderGradient(topicName);
            }}
          />
        ) : (
          <div
            className="w-full h-full"
            style={{ background: generatePlaceholderGradient(topicName) }}
          />
        )}
        <div className="absolute inset-0 bg-gradient-to-t from-black/50 to-transparent" />
      </div>

      <div className="flex items-start justify-between gap-4 mb-3">
        <div className="flex-1">
          <div className="flex items-center gap-2 mb-2">
            <motion.span
              className={`px-2 py-1 text-xs font-medium rounded-full border ${relevanceColors[relevance]}`}
              animate={relevance === 'high' ? {
                boxShadow: [
                  "0 0 10px rgba(239, 68, 68, 0.3)",
                  "0 0 20px rgba(239, 68, 68, 0.5)",
                  "0 0 10px rgba(239, 68, 68, 0.3)",
                ]
              } : {}}
              transition={{ duration: 2, repeat: Infinity }}
            >
              {relevance.toUpperCase()}
            </motion.span>
            <span className="text-sm text-gray-500 dark:text-gray-400">
              {topicName}
            </span>
            <span className="text-xs text-gray-400 dark:text-gray-500">
              {formatDistanceToNow(parseLocalDate(briefing.created_at), { addSuffix: true })}
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

      {briefing.detailed_content && expanded && (
        <div className="mb-4 p-4 bg-gray-50 dark:bg-gray-800/50 rounded-lg border border-gray-200 dark:border-gray-700">
          <h4 className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2 flex items-center gap-2">
            <Sparkles className="w-4 h-4 text-primary-600 dark:text-primary-400" />
            Detailed Research
          </h4>
          <p className="text-sm text-gray-700 dark:text-gray-300 leading-relaxed whitespace-pre-line">
            {briefing.detailed_content}
          </p>
        </div>
      )}

      {sources.length > 0 && (
        <div className="mb-4">
          <h4 className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            Sources ({sources.length})
          </h4>
          <ul className="space-y-1">
            {sources.slice(0, expanded ? undefined : 3).map((source, idx) => {
              const { href, text } = getSourceDisplay(source);
              return (
                <li key={idx}>
                  {href ? (
                    <a
                      href={href}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="text-sm text-primary-600 dark:text-primary-400 hover:underline flex items-center gap-1"
                    >
                      <ExternalLink className="w-3 h-3" />
                      {text}
                    </a>
                  ) : (
                    <span className="text-sm text-gray-600 dark:text-gray-400 flex items-center gap-1">
                      {text}
                    </span>
                  )}
                </li>
              );
            })}
          </ul>
          {sources.length > 3 && !expanded && (
            <button
              onClick={() => setExpanded(true)}
              className="text-sm text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300 mt-1"
            >
              +{sources.length - 3} more sources
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
          {/* Thumbs up/down buttons commented out - not currently used for anything
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
          */}
          <button
            onClick={onBookmark}
            className={`p-2 rounded-lg transition-colors ${
              isBookmarked
                ? 'bg-yellow-100 dark:bg-yellow-900/30 text-yellow-600 dark:text-yellow-400'
                : 'hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-600 dark:text-gray-400'
            }`}
            aria-label={isBookmarked ? 'Remove bookmark' : 'Add bookmark'}
          >
            <Bookmark className={`w-4 h-4 ${isBookmarked ? 'fill-current' : ''}`} />
          </button>
          <button
            onClick={onOpenChat}
            className={`p-2 rounded-lg transition-colors ${
              hasChat
                ? 'bg-primary-100 dark:bg-primary-900/30 text-primary-600 dark:text-primary-400'
                : 'hover:bg-primary-100 dark:hover:bg-primary-900/30 text-gray-600 dark:text-gray-400 hover:text-primary-600 dark:hover:text-primary-400'
            }`}
            aria-label="Chat about this briefing"
          >
            <MessageCircle className={`w-4 h-4 ${hasChat ? 'fill-current' : ''}`} />
          </button>
        </div>

        {(sources.length > 3 || briefing.detailed_content) && (
          <button
            onClick={() => setExpanded(!expanded)}
            className="flex items-center gap-1.5 text-sm px-3 py-1.5 rounded-lg bg-primary-500/20 text-primary-400 hover:bg-primary-500/30 hover:text-primary-300 transition-colors"
          >
            {expanded ? (
              <>
                <ChevronUp className="w-4 h-4" />
                Show less
              </>
            ) : (
              <>
                <ChevronDown className="w-4 h-4" />
                {briefing.detailed_content ? 'Show research' : 'Show more sources'}
              </>
            )}
          </button>
        )}
      </div>
    </motion.div>

    {/* Delete Confirmation Dialog */}
    <DeleteConfirmDialog
      isOpen={showDeleteConfirm}
      isBookmarked={isBookmarked ?? false}
      onConfirm={confirmDelete}
      onCancel={() => setShowDeleteConfirm(false)}
    />
    </>
  );
}
