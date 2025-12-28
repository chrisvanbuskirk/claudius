import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { /* ThumbsUp, ThumbsDown, */ ExternalLink, ChevronDown, ChevronUp, Sparkles, MessageCircle, Bookmark, X, AlertTriangle, Printer, Copy, Check, Download, Share2 } from 'lucide-react';
import { formatDistanceToNow, parseISO } from 'date-fns';
import { convertFileSrc } from '@tauri-apps/api/core';
import ReactMarkdown from 'react-markdown';
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

// Simple markdown to HTML conversion for print
function markdownToHtml(markdown: string): string {
  return markdown
    // Bold: **text** or __text__
    .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
    .replace(/__(.+?)__/g, '<strong>$1</strong>')
    // Italic: *text* or _text_
    .replace(/\*(.+?)\*/g, '<em>$1</em>')
    .replace(/_(.+?)_/g, '<em>$1</em>')
    // Headers: ## Header
    .replace(/^### (.+)$/gm, '<h4>$1</h4>')
    .replace(/^## (.+)$/gm, '<h3>$1</h3>')
    .replace(/^# (.+)$/gm, '<h2>$1</h2>')
    // Bullet lists: - item or * item
    .replace(/^[-*] (.+)$/gm, '<li>$1</li>')
    // Wrap consecutive <li> in <ul>
    .replace(/(<li>.*<\/li>\n?)+/g, '<ul>$&</ul>')
    // Links: [text](url)
    .replace(/\[(.+?)\]\((.+?)\)/g, '<a href="$2" target="_blank">$1</a>')
    // Line breaks
    .replace(/\n/g, '<br>');
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

  // Default values for optional fields (must be before handlers that use them)
  const relevance = (briefing.relevance || 'medium') as 'high' | 'medium' | 'low';
  const sources = briefing.sources || [];
  const topicName = briefing.topic_name || 'General';

  const handleDelete = () => {
    setShowDeleteConfirm(true);
  };

  const confirmDelete = () => {
    setShowDeleteConfirm(false);
    onDelete?.();
  };

  const handlePrint = async () => {
    console.log('[Print] handlePrint called');

    const formattedDate = new Date(briefing.created_at).toLocaleDateString('en-US', {
      weekday: 'long',
      year: 'numeric',
      month: 'long',
      day: 'numeric',
    });

    const sourcesHtml = sources.length > 0
      ? `<div class="sources">
          <h3>Sources</h3>
          <ul>
            ${sources.map(source => {
              const { href, text } = getSourceDisplay(source);
              return href
                ? `<li><a href="${href}" target="_blank">${text}</a></li>`
                : `<li>${text}</li>`;
            }).join('')}
          </ul>
        </div>`
      : '';

    const detailedContentHtml = briefing.detailed_content
      ? `<div class="detailed-content">
          <h3>Detailed Research</h3>
          <div>${markdownToHtml(briefing.detailed_content)}</div>
        </div>`
      : '';

    const suggestedNextHtml = briefing.suggested_next
      ? `<div class="suggested-next">
          <h3>Suggested Next Step</h3>
          <p>${briefing.suggested_next}</p>
        </div>`
      : '';

    // Generate image HTML if available
    const imageHtml = briefing.image_path
      ? `<div class="header-image">
          <img src="file://${briefing.image_path}" alt="" />
        </div>`
      : '';

    const printContent = `<!DOCTYPE html>
<html>
<head>
<title>${briefing.title} - Claudius Briefing</title>
<style>
* { box-sizing: border-box; }
body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
  line-height: 1.6;
  max-width: 800px;
  margin: 0 auto;
  padding: 40px 20px;
  color: #1a1a1a;
}
.header-image {
  margin: -40px -20px 24px -20px;
  max-height: 300px;
  overflow: hidden;
}
.header-image img {
  width: 100%;
  height: 300px;
  object-fit: cover;
}
.header {
  border-bottom: 2px solid #e5e7eb;
  padding-bottom: 20px;
  margin-bottom: 24px;
}
.meta {
  display: flex;
  gap: 12px;
  align-items: center;
  margin-bottom: 12px;
  font-size: 14px;
  color: #6b7280;
}
.relevance {
  display: inline-block;
  padding: 2px 8px;
  border-radius: 12px;
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
}
.relevance.high { background: #fee2e2; color: #dc2626; }
.relevance.medium { background: #fef3c7; color: #d97706; }
.relevance.low { background: #dbeafe; color: #2563eb; }
h1 {
  font-size: 28px;
  font-weight: 700;
  margin: 0 0 8px 0;
  color: #111827;
}
.summary {
  font-size: 18px;
  color: #374151;
  margin-bottom: 24px;
}
h3 {
  font-size: 16px;
  font-weight: 600;
  color: #374151;
  margin: 24px 0 12px 0;
  padding-bottom: 8px;
  border-bottom: 1px solid #e5e7eb;
}
.detailed-content p, .suggested-next p {
  margin: 0;
  color: #4b5563;
}
.sources ul {
  margin: 0;
  padding-left: 20px;
}
.sources li {
  margin-bottom: 4px;
}
.sources a {
  color: #6366f1;
  text-decoration: none;
}
.suggested-next {
  background: #f0fdf4;
  padding: 16px;
  border-radius: 8px;
  border: 1px solid #bbf7d0;
}
.footer {
  margin-top: 40px;
  padding-top: 20px;
  border-top: 1px solid #e5e7eb;
  font-size: 12px;
  color: #9ca3af;
  text-align: center;
}
@media print {
  body { padding: 20px; }
  .suggested-next { break-inside: avoid; }
}
</style>
</head>
<body>
${imageHtml}
<div class="header">
  <div class="meta">
    <span class="relevance ${relevance}">${relevance.toUpperCase()}</span>
    <span>${topicName}</span>
    <span>•</span>
    <span>${formattedDate}</span>
  </div>
  <h1>${briefing.title}</h1>
</div>
<div class="summary">${briefing.summary}</div>
${detailedContentHtml}
${sourcesHtml}
${suggestedNextHtml}
<div class="footer">
  Generated by Claudius • Printed ${new Date().toLocaleDateString()}
</div>
</body>
</html>`;

    try {
      // Use Tauri invoke to call a Rust command that opens the print preview
      const { invoke } = await import('@tauri-apps/api/core');
      console.log('[Print] Calling print_card command...');
      await invoke('print_card', { html: printContent });
      console.log('[Print] Command completed');
    } catch (error) {
      console.error('[Print] Tauri command failed:', error);
      // Fallback: copy to clipboard and show message
      try {
        await navigator.clipboard.writeText(briefing.summary + '\n\n' + (briefing.detailed_content || ''));
        alert('Print is not available. Content copied to clipboard - paste into a document to print.');
      } catch {
        alert('Print is not available in the desktop app.');
      }
    }
  };

  // State for copy button feedback
  const [copied, setCopied] = useState(false);

  // Generate markdown content for copying/exporting
  const generateMarkdown = () => {
    const formattedDate = new Date(briefing.created_at).toLocaleDateString('en-US', {
      weekday: 'long',
      year: 'numeric',
      month: 'long',
      day: 'numeric',
    });

    let markdown = `# ${briefing.title}\n\n`;
    markdown += `**${topicName}** • ${formattedDate} • ${relevance.toUpperCase()} relevance\n\n`;
    markdown += `## Summary\n\n${briefing.summary}\n\n`;
    
    if (briefing.detailed_content) {
      markdown += `## Detailed Research\n\n${briefing.detailed_content}\n\n`;
    }
    
    if (sources.length > 0) {
      markdown += `## Sources\n\n`;
      sources.forEach((source) => {
        const { href, text } = getSourceDisplay(source);
        if (href) {
          markdown += `- [${text}](${href})\n`;
        } else {
          markdown += `- ${text}\n`;
        }
      });
      markdown += '\n';
    }
    
    if (briefing.suggested_next) {
      markdown += `## Suggested Next Step\n\n${briefing.suggested_next}\n\n`;
    }
    
    markdown += `---\n*Generated by Claudius*\n`;
    
    return markdown;
  };

  // Copy to clipboard
  const handleCopy = async () => {
    try {
      const markdown = generateMarkdown();
      await navigator.clipboard.writeText(markdown);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (error) {
      console.error('Failed to copy:', error);
    }
  };

  // Export to markdown file
  const handleExport = async (format: 'markdown') => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      
      const markdown = generateMarkdown();
      const filename = `${briefing.title.replace(/[^a-z0-9]/gi, '_').toLowerCase()}.md`;
      
      await invoke('export_card', {
        content: markdown,
        defaultFilename: filename,
        fileType: format,
      });
    } catch (error) {
      console.error('Failed to export:', error);
      // Fallback: copy to clipboard
      const markdown = generateMarkdown();
      await navigator.clipboard.writeText(markdown);
      alert('Export not available. Content copied to clipboard instead.');
    }
  };

  // Native share
  const handleShare = async () => {
    try {
      if (navigator.share) {
        await navigator.share({
          title: briefing.title,
          text: `${briefing.summary}\n\n${briefing.detailed_content || ''}`,
        });
      } else {
        // Fallback: copy to clipboard
        await handleCopy();
      }
    } catch (error) {
      // User cancelled share or share failed
      if ((error as Error).name !== 'AbortError') {
        console.error('Failed to share:', error);
      }
    }
  };

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
          <div className="text-sm text-gray-700 dark:text-gray-300 leading-relaxed prose prose-sm dark:prose-invert max-w-none">
            <ReactMarkdown>{briefing.detailed_content}</ReactMarkdown>
          </div>
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
          {/* Copy to clipboard */}
          <button
            onClick={handleCopy}
            className={`p-2 rounded-lg transition-colors ${
              copied
                ? 'bg-green-100 dark:bg-green-900/30 text-green-600 dark:text-green-400'
                : 'hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-600 dark:text-gray-400'
            }`}
            aria-label="Copy to clipboard"
            title="Copy to clipboard"
          >
            {copied ? <Check className="w-4 h-4" /> : <Copy className="w-4 h-4" />}
          </button>
          {/* Export as Markdown */}
          <button
            onClick={() => handleExport('markdown')}
            className="p-2 rounded-lg transition-colors hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-600 dark:text-gray-400"
            aria-label="Export as Markdown"
            title="Export as Markdown"
          >
            <Download className="w-4 h-4" />
          </button>
          {/* Native share */}
          {typeof navigator !== 'undefined' && 'share' in navigator && (
            <button
              onClick={handleShare}
              className="p-2 rounded-lg transition-colors hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-600 dark:text-gray-400"
              aria-label="Share"
              title="Share"
            >
              <Share2 className="w-4 h-4" />
            </button>
          )}
          <button
            onClick={handlePrint}
            className="p-2 rounded-lg transition-colors hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-600 dark:text-gray-400"
            aria-label="Print this card"
          >
            <Printer className="w-4 h-4" />
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
