import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { AlertTriangle, X, ExternalLink } from 'lucide-react';

interface ResearchLogRecord {
  id: number;
  briefing_id: number | null;
  log_type: string;
  topic: string | null;
  tool_name: string | null;
  input_summary: string | null;
  output_summary: string | null;
  duration_ms: number | null;
  tokens_used: number | null;
  success: boolean;
  error_code: string | null;
  error_message: string | null;
  user_action_required: boolean;
  created_at: string;
}

// Map error codes to user-friendly messages and actions
const errorMessages: Record<string, { message: string; action?: string; link?: string }> = {
  invalid_api_key: {
    message: 'Your API key is invalid or has been revoked.',
    action: 'Go to Settings to update your API key.',
    link: '/settings',
  },
  budget_exceeded: {
    message: 'Your Anthropic API budget has been exceeded.',
    action: 'Add credits to your account.',
    link: 'https://console.anthropic.com',
  },
  mcp_connection_failed: {
    message: 'Failed to connect to an MCP server.',
    action: 'Check your MCP server configuration in Settings.',
    link: '/settings',
  },
};

export function ActionableErrorsAlert() {
  const [errors, setErrors] = useState<ResearchLogRecord[]>([]);
  const [dismissed, setDismissed] = useState<Set<number>>(new Set());
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadErrors();
  }, []);

  async function loadErrors() {
    try {
      const result = await invoke<ResearchLogRecord[]>('get_actionable_errors', { limit: 5 });
      setErrors(result);
    } catch (err) {
      console.error('Failed to load actionable errors:', err);
    } finally {
      setLoading(false);
    }
  }

  function handleDismiss(id: number) {
    setDismissed((prev) => new Set(prev).add(id));
  }

  // Filter out dismissed errors and get unique errors by error_code
  const visibleErrors = errors
    .filter((e) => !dismissed.has(e.id))
    .reduce((acc, error) => {
      // Only keep the most recent error of each type
      const existing = acc.find((e) => e.error_code === error.error_code);
      if (!existing) {
        acc.push(error);
      }
      return acc;
    }, [] as ResearchLogRecord[]);

  if (loading || visibleErrors.length === 0) {
    return null;
  }

  return (
    <div className="space-y-3 mb-6">
      {visibleErrors.map((error) => {
        const errorInfo = error.error_code ? errorMessages[error.error_code] : null;
        const message = errorInfo?.message || error.error_message || 'An error occurred that requires your attention.';
        const action = errorInfo?.action;
        const link = errorInfo?.link;
        const isExternalLink = link?.startsWith('http');

        return (
          <div
            key={error.id}
            className="card p-4 bg-amber-50 dark:bg-amber-900/20 border-amber-200 dark:border-amber-800"
          >
            <div className="flex items-start gap-3">
              <AlertTriangle className="w-5 h-5 text-amber-600 dark:text-amber-400 flex-shrink-0 mt-0.5" />
              <div className="flex-1 min-w-0">
                <h3 className="font-semibold text-amber-900 dark:text-amber-300 mb-1">
                  Action Required
                </h3>
                <p className="text-sm text-amber-800 dark:text-amber-400 mb-2">
                  {message}
                </p>
                {action && link && (
                  <div className="flex items-center gap-1">
                    {isExternalLink ? (
                      <a
                        href={link}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="text-sm text-amber-700 dark:text-amber-300 hover:underline flex items-center gap-1"
                      >
                        {action}
                        <ExternalLink className="w-3 h-3" />
                      </a>
                    ) : (
                      <a
                        href={`#${link}`}
                        className="text-sm text-amber-700 dark:text-amber-300 hover:underline"
                      >
                        {action}
                      </a>
                    )}
                  </div>
                )}
                <p className="text-xs text-amber-600/70 dark:text-amber-500/70 mt-2">
                  {new Date(error.created_at).toLocaleString()}
                </p>
              </div>
              <button
                onClick={() => handleDismiss(error.id)}
                className="p-1 rounded hover:bg-amber-200/50 dark:hover:bg-amber-800/50 text-amber-600 dark:text-amber-400"
                title="Dismiss"
              >
                <X className="w-4 h-4" />
              </button>
            </div>
          </div>
        );
      })}
    </div>
  );
}
