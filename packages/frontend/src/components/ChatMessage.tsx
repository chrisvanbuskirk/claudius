import { useState } from 'react';
import { motion } from 'framer-motion';
import { formatDistanceToNow } from 'date-fns';
import { User, Sparkles, Copy, Check } from 'lucide-react';
import ReactMarkdown from 'react-markdown';
import type { ChatMessage as ChatMessageType } from '../types';

interface ChatMessageProps {
  message: ChatMessageType;
}

export function ChatMessage({ message }: ChatMessageProps) {
  const isUser = message.role === 'user';
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(message.content);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error('Failed to copy:', err);
    }
  };

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.2 }}
      className={`group flex gap-3 ${isUser ? 'flex-row-reverse' : 'flex-row'}`}
    >
      {/* Avatar */}
      <div
        className={`flex-shrink-0 w-8 h-8 rounded-full flex items-center justify-center ${
          isUser
            ? 'bg-primary-500/20 text-primary-400'
            : 'bg-purple-500/20 text-purple-400'
        }`}
      >
        {isUser ? (
          <User className="w-4 h-4" />
        ) : (
          <Sparkles className="w-4 h-4" />
        )}
      </div>

      {/* Message bubble */}
      <div className="flex-1 max-w-[80%]">
        <div
          className={`relative rounded-2xl px-4 py-3 ${
            isUser
              ? 'bg-primary-600 text-white rounded-br-sm'
              : 'bg-gray-100 dark:bg-gray-700/50 text-gray-900 dark:text-gray-100 rounded-bl-sm'
          }`}
        >
          {isUser ? (
            <p className="text-sm whitespace-pre-wrap leading-relaxed">
              {message.content}
            </p>
          ) : (
            <div className="text-sm leading-relaxed prose prose-sm dark:prose-invert max-w-none prose-p:my-2 prose-headings:my-2 prose-ul:my-2 prose-ol:my-2 prose-li:my-0.5 prose-pre:my-2 prose-pre:bg-gray-800 prose-pre:text-gray-100 prose-code:text-purple-400 prose-code:bg-gray-800/50 prose-code:px-1 prose-code:py-0.5 prose-code:rounded prose-code:before:content-none prose-code:after:content-none">
              <ReactMarkdown>{message.content}</ReactMarkdown>
            </div>
          )}
        </div>

        {/* Footer with timestamp and copy button */}
        <div
          className={`flex items-center gap-2 mt-1 px-1 ${
            isUser ? 'justify-end' : 'justify-start'
          }`}
        >
          <span
            className={`text-xs ${
              isUser ? 'text-gray-400' : 'text-gray-500 dark:text-gray-400'
            }`}
          >
            {formatDistanceToNow(new Date(message.created_at), { addSuffix: true })}
            {message.tokens_used && !isUser && (
              <span className="ml-2 opacity-60">
                ({message.tokens_used} tokens)
              </span>
            )}
          </span>

          {/* Copy button - always visible for assistant, on hover for user */}
          <button
            onClick={handleCopy}
            className={`p-1 rounded transition-all ${
              isUser
                ? 'opacity-0 group-hover:opacity-100 text-gray-400 hover:text-gray-300 hover:bg-gray-700/50'
                : 'text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600/50'
            }`}
            title="Copy message"
          >
            {copied ? (
              <Check className="w-3.5 h-3.5 text-green-500" />
            ) : (
              <Copy className="w-3.5 h-3.5" />
            )}
          </button>
        </div>
      </div>
    </motion.div>
  );
}
