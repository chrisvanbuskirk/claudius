import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface Briefing {
  id: number;
  date: string;
  title: string;
  cards: string;
}

interface BriefingCard {
  topic: string;
  summary: string;
  content: string;
  sources?: string[];
}

interface Topic {
  id: string;
  name: string;
  description: string;
  enabled: boolean;
}

interface SchedulerStatus {
  running: boolean;
  schedule: string | null;
}

export function PopoverApp() {
  const [briefings, setBriefings] = useState<Briefing[]>([]);
  const [topics, setTopics] = useState<Topic[]>([]);
  const [schedulerStatus, setSchedulerStatus] = useState<SchedulerStatus | null>(null);
  const [nextRunTime, setNextRunTime] = useState<string | null>(null);
  const [expandedId, setExpandedId] = useState<number | null>(null);
  const [isRunning, setIsRunning] = useState(false);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadData();
  }, []);

  async function loadData() {
    try {
      const [briefingsResult, statusResult, nextRun, topicsResult] = await Promise.all([
        invoke<Briefing[]>('get_todays_briefings'),
        invoke<SchedulerStatus>('get_scheduler_status'),
        invoke<string | null>('get_next_run_time'),
        invoke<Topic[]>('get_topics'),
      ]);
      setBriefings(briefingsResult);
      setSchedulerStatus(statusResult);
      setNextRunTime(nextRun);
      setTopics(topicsResult.filter(t => t.enabled));
    } catch (err) {
      console.error('Failed to load data:', err);
    } finally {
      setLoading(false);
    }
  }

  async function handleRunNow() {
    // Guard against concurrent clicks
    if (isRunning) return;

    setIsRunning(true);
    try {
      await invoke('run_research_now');
      // Reload briefings after research completes
      const newBriefings = await invoke<Briefing[]>('get_todays_briefings');
      setBriefings(newBriefings);
    } catch (err) {
      console.error('Research failed:', err);
    } finally {
      setIsRunning(false);
    }
  }

  async function handleOpenSettings() {
    await invoke('open_settings_window');
  }

  async function handleOpenMainWindow() {
    await invoke('open_main_window');
  }

  async function handleViewBriefing(_id: number) {
    await invoke('open_main_window');
  }

  function parseCards(cards: string): BriefingCard[] {
    try {
      const parsed = JSON.parse(cards);
      if (Array.isArray(parsed)) {
        return parsed;
      }
    } catch {
      // Fall back to empty
    }
    return [];
  }

  return (
    <div className="w-80 h-[520px] bg-white dark:bg-gray-900 rounded-lg shadow-2xl overflow-hidden flex flex-col border border-gray-200 dark:border-gray-700">
      {/* Header with branding - matches main app sidebar */}
      <div className="p-3 bg-gradient-to-r from-blue-600 to-blue-700 text-white">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <div className="w-7 h-7 bg-white/20 rounded-lg flex items-center justify-center">
              <svg className="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <path d="m12 3-1.912 5.813a2 2 0 0 1-1.275 1.275L3 12l5.813 1.912a2 2 0 0 1 1.275 1.275L12 21l1.912-5.813a2 2 0 0 1 1.275-1.275L21 12l-5.813-1.912a2 2 0 0 1-1.275-1.275L12 3Z"/>
                <path d="M5 3v4"/>
                <path d="M19 17v4"/>
                <path d="M3 5h4"/>
                <path d="M17 19h4"/>
              </svg>
            </div>
            <div>
              <span className="font-semibold">Claudius</span>
              <p className="text-[10px] text-blue-200">AI Research Assistant</p>
            </div>
          </div>
          <div className="flex gap-1">
            <button
              onClick={handleRunNow}
              disabled={isRunning}
              className="p-1.5 rounded hover:bg-white/20 disabled:opacity-50 transition-colors"
              title="Run Research Now"
            >
              {isRunning ? (
                <svg className="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                  <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                  <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
                </svg>
              ) : (
                <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                  <path d="M6.3 2.8A1.5 1.5 0 004 4.11v11.78a1.5 1.5 0 002.3 1.27l9.344-5.891a1.5 1.5 0 000-2.538L6.3 2.8z" />
                </svg>
              )}
            </button>
            <button
              onClick={handleOpenSettings}
              className="p-1.5 rounded hover:bg-white/20 transition-colors"
              title="Settings"
            >
              <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
              </svg>
            </button>
          </div>
        </div>
        {/* Scheduler status */}
        <div className="mt-2 flex items-center gap-2 text-xs text-blue-100">
          <div className={`w-1.5 h-1.5 rounded-full ${schedulerStatus?.running ? 'bg-green-400' : 'bg-gray-400'}`} />
          <span>{nextRunTime ? `Next briefing: ${nextRunTime}` : 'Scheduler paused'}</span>
        </div>
      </div>

      {/* Topics Overview */}
      {topics.length > 0 && (
        <div className="px-3 py-2 bg-gray-50 dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700">
          <div className="text-xs text-gray-500 dark:text-gray-400 mb-1.5">Tracking {topics.length} topic{topics.length !== 1 ? 's' : ''}</div>
          <div className="flex flex-wrap gap-1">
            {topics.slice(0, 5).map((topic) => (
              <span
                key={topic.id}
                className="px-2 py-0.5 text-xs bg-blue-100 dark:bg-blue-900 text-blue-700 dark:text-blue-300 rounded-full truncate max-w-[100px]"
                title={topic.description}
              >
                {topic.name}
              </span>
            ))}
            {topics.length > 5 && (
              <span className="px-2 py-0.5 text-xs text-gray-500 dark:text-gray-400">
                +{topics.length - 5} more
              </span>
            )}
          </div>
        </div>
      )}

      {/* Briefings List */}
      <div className="flex-1 overflow-y-auto">
        {loading ? (
          <div className="flex items-center justify-center h-full">
            <svg className="w-6 h-6 animate-spin text-gray-400" fill="none" viewBox="0 0 24 24">
              <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
              <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
            </svg>
          </div>
        ) : briefings.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-gray-500 dark:text-gray-400 px-4 text-center">
            <svg className="w-12 h-12 mb-3 opacity-50" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
            </svg>
            <p className="text-sm font-medium">No briefings yet today</p>
            <p className="text-xs mt-1 text-gray-400">
              {topics.length > 0
                ? 'Click the play button to generate a briefing'
                : 'Add some topics in settings first'}
            </p>
            {topics.length > 0 && (
              <button
                onClick={handleRunNow}
                disabled={isRunning}
                className="mt-3 px-4 py-1.5 text-sm bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50 transition-colors"
              >
                {isRunning ? 'Generating...' : 'Generate Briefing'}
              </button>
            )}
          </div>
        ) : (
          <div>
            <div className="px-3 py-2 text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider flex items-center justify-between">
              <span>Today's Briefings</span>
              <span className="text-gray-400">{briefings.length}</span>
            </div>
            {briefings.map((briefing) => {
              const cards = parseCards(briefing.cards);
              const cardCount = cards.length;

              return (
                <div key={briefing.id} className="border-b border-gray-100 dark:border-gray-800">
                  <button
                    onClick={() => setExpandedId(expandedId === briefing.id ? null : briefing.id)}
                    className="w-full px-3 py-2.5 text-left hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors"
                  >
                    <div className="flex items-center justify-between">
                      <div className="flex-1 min-w-0">
                        <h3 className="font-medium text-sm text-gray-900 dark:text-gray-100 truncate">
                          {briefing.title}
                        </h3>
                        <p className="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
                          {cardCount} card{cardCount !== 1 ? 's' : ''} • {new Date(briefing.date).toLocaleTimeString([], { hour: 'numeric', minute: '2-digit' })}
                        </p>
                      </div>
                      <svg
                        className={`w-4 h-4 text-gray-400 transition-transform ml-2 flex-shrink-0 ${expandedId === briefing.id ? 'rotate-180' : ''}`}
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke="currentColor"
                      >
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
                      </svg>
                    </div>
                  </button>
                  {expandedId === briefing.id && cards.length > 0 && (
                    <div className="px-3 pb-3 space-y-2">
                      {cards.slice(0, 3).map((card, idx) => (
                        <div key={idx} className="p-2 bg-gray-50 dark:bg-gray-800 rounded text-xs">
                          <div className="font-medium text-gray-700 dark:text-gray-300 mb-1">{card.topic}</div>
                          <p className="text-gray-600 dark:text-gray-400 line-clamp-2">{card.summary}</p>
                        </div>
                      ))}
                      {cards.length > 3 && (
                        <p className="text-xs text-gray-400 dark:text-gray-500 text-center">
                          +{cards.length - 3} more cards
                        </p>
                      )}
                      <button
                        onClick={() => handleViewBriefing(briefing.id)}
                        className="w-full mt-1 py-1.5 text-xs text-blue-600 hover:text-blue-700 dark:text-blue-400 font-medium bg-blue-50 dark:bg-blue-900/30 rounded hover:bg-blue-100 dark:hover:bg-blue-900/50 transition-colors"
                      >
                        View Full Briefing
                      </button>
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        )}
      </div>

      {/* Footer */}
      <div className="p-2 border-t border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800">
        <button
          onClick={handleOpenMainWindow}
          className="w-full px-3 py-2 text-sm font-medium text-gray-700 dark:text-gray-200 hover:text-gray-900 dark:hover:text-white bg-white dark:bg-gray-700 hover:bg-gray-100 dark:hover:bg-gray-600 rounded-md border border-gray-200 dark:border-gray-600 transition-colors"
        >
          Open Full App
        </button>
      </div>
    </div>
  );
}
