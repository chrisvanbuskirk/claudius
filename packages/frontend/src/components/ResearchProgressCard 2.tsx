import { motion } from 'framer-motion';
import { Loader2, CheckCircle2, XCircle, Database, Brain } from 'lucide-react';
import type { ResearchProgressState } from '../hooks/useResearchProgress';

interface ResearchProgressCardProps {
  progress: ResearchProgressState;
}

export function ResearchProgressCard({ progress }: ResearchProgressCardProps) {
  if (!progress.isRunning && progress.currentPhase !== 'complete') {
    return null;
  }

  const getPhaseMessage = () => {
    switch (progress.currentPhase) {
      case 'starting':
        return 'Starting research...';
      case 'researching':
        return `Researching: ${progress.currentTopicName}`;
      case 'saving':
        return 'Saving research results to database...';
      case 'complete':
        return progress.error ? 'Research failed' : 'Research complete!';
      default:
        return 'Processing...';
    }
  };

  const getPhaseIcon = () => {
    switch (progress.currentPhase) {
      case 'saving':
        return <Database className="w-5 h-5 animate-pulse text-primary-500" />;
      case 'complete':
        return progress.error ? (
          <XCircle className="w-5 h-5 text-red-500" />
        ) : (
          <CheckCircle2 className="w-5 h-5 text-green-500" />
        );
      default:
        return <Brain className="w-5 h-5 animate-pulse text-primary-500" />;
    }
  };

  const progressPercentage = progress.totalTopics > 0
    ? Math.round((progress.topicsCompleted.length / progress.totalTopics) * 100)
    : 0;

  return (
    <motion.div
      className="glass-card p-6 mb-6 border-primary-500/30"
      initial={{ opacity: 0, y: -10 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -10 }}
    >
      {/* Header */}
      <div className="flex items-center gap-3 mb-4">
        {getPhaseIcon()}
        <div className="flex-1">
          <h3 className="font-semibold text-gray-900 dark:text-white mb-1">
            {getPhaseMessage()}
          </h3>
          {progress.totalTopics > 0 && (
            <p className="text-sm text-gray-600 dark:text-gray-400">
              {progress.topicsCompleted.length} of {progress.totalTopics} topics completed ({progressPercentage}%)
            </p>
          )}
        </div>
        {progress.isRunning && (
          <Loader2 className="w-5 h-5 animate-spin text-primary-500" />
        )}
      </div>

      {/* Progress bar */}
      {progress.totalTopics > 0 && (
        <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2 mb-4">
          <motion.div
            className="bg-gradient-to-r from-primary-500 to-purple-500 h-2 rounded-full"
            initial={{ width: 0 }}
            animate={{ width: `${progressPercentage}%` }}
            transition={{ duration: 0.5 }}
          />
        </div>
      )}

      {/* Completed Topics */}
      {progress.topicsCompleted.length > 0 && (
        <div>
          <h4 className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            Completed Topics
          </h4>
          <div className="space-y-1">
            {progress.topicsCompleted.map((topic, idx) => (
              <div
                key={`${topic.topicName}-${idx}`}
                className="flex items-center gap-2 text-sm"
              >
                <CheckCircle2 className="w-4 h-4 text-green-500" />
                <span className="text-gray-700 dark:text-gray-300">
                  {topic.topicName}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Error Display */}
      {progress.error && (
        <div className="mt-4 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
          <p className="text-sm text-red-700 dark:text-red-400">
            {progress.error}
          </p>
        </div>
      )}
    </motion.div>
  );
}
