import { useState } from 'react';
import { Save, Plus, X, Trash2, CheckCircle2, Loader2, Play } from 'lucide-react';
import { useTopics, useMCPServers, useSettings } from '../hooks/useTauri';

type Tab = 'interests' | 'mcp' | 'research';

export function SettingsPage() {
  const [activeTab, setActiveTab] = useState<Tab>('interests');

  const tabs: { id: Tab; label: string }[] = [
    { id: 'interests', label: 'Interests' },
    { id: 'mcp', label: 'MCP Servers' },
    { id: 'research', label: 'Research Settings' },
  ];

  return (
    <div>
      <div className="mb-8">
        <h1 className="text-3xl font-bold text-gray-900 dark:text-white mb-1">
          Settings
        </h1>
        <p className="text-gray-600 dark:text-gray-400">
          Configure your research assistant
        </p>
      </div>

      <div className="card mb-6">
        <div className="border-b border-gray-200 dark:border-gray-700">
          <nav className="flex -mb-px">
            {tabs.map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                className={`px-6 py-4 text-sm font-medium border-b-2 transition-colors ${
                  activeTab === tab.id
                    ? 'border-primary-600 text-primary-600 dark:text-primary-400'
                    : 'border-transparent text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300 hover:border-gray-300 dark:hover:border-gray-600'
                }`}
              >
                {tab.label}
              </button>
            ))}
          </nav>
        </div>

        <div className="p-6">
          {activeTab === 'interests' && <InterestsTab />}
          {activeTab === 'mcp' && <MCPServersTab />}
          {activeTab === 'research' && <ResearchSettingsTab />}
        </div>
      </div>
    </div>
  );
}

function InterestsTab() {
  const { topics, loading, addTopic, updateTopic, deleteTopic } = useTopics();
  const [newTopicName, setNewTopicName] = useState('');
  const [newTopicDescription, setNewTopicDescription] = useState('');
  const [showAddForm, setShowAddForm] = useState(false);
  const [saving, setSaving] = useState(false);

  const handleAddTopic = async () => {
    if (!newTopicName.trim()) return;

    setSaving(true);
    try {
      await addTopic(newTopicName, newTopicDescription || undefined);
      setNewTopicName('');
      setNewTopicDescription('');
      setShowAddForm(false);
    } finally {
      setSaving(false);
    }
  };

  const handleToggleTopic = async (topicId: string, enabled: boolean) => {
    await updateTopic(topicId, undefined, undefined, enabled);
  };

  const handleDeleteTopic = async (topicId: string) => {
    if (confirm('Are you sure you want to delete this topic?')) {
      await deleteTopic(topicId);
    }
  };

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <div>
          <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-1">
            Research Topics
          </h2>
          <p className="text-sm text-gray-600 dark:text-gray-400">
            Add topics you want to stay informed about
          </p>
        </div>
        {!showAddForm && (
          <button
            onClick={() => setShowAddForm(true)}
            className="btn btn-primary flex items-center gap-2"
          >
            <Plus className="w-4 h-4" />
            Add Topic
          </button>
        )}
      </div>

      {showAddForm && (
        <div className="mb-6 p-4 bg-gray-50 dark:bg-gray-800/50 rounded-lg border border-gray-200 dark:border-gray-700">
          <div className="flex items-start justify-between mb-3">
            <h3 className="font-medium text-gray-900 dark:text-white">New Topic</h3>
            <button
              onClick={() => setShowAddForm(false)}
              className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
            >
              <X className="w-5 h-5" />
            </button>
          </div>
          <div className="space-y-3">
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                Topic Name
              </label>
              <input
                type="text"
                value={newTopicName}
                onChange={(e) => setNewTopicName(e.target.value)}
                placeholder="e.g., Machine Learning, Climate Change"
                className="input w-full"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                Description (optional)
              </label>
              <textarea
                value={newTopicDescription}
                onChange={(e) => setNewTopicDescription(e.target.value)}
                placeholder="Describe what aspects you're interested in..."
                rows={3}
                className="input w-full resize-none"
              />
            </div>
            <div className="flex gap-2">
              <button
                onClick={handleAddTopic}
                disabled={!newTopicName.trim() || saving}
                className="btn btn-primary flex items-center gap-2"
              >
                {saving ? (
                  <Loader2 className="w-4 h-4 animate-spin" />
                ) : (
                  <Plus className="w-4 h-4" />
                )}
                Add Topic
              </button>
              <button
                onClick={() => setShowAddForm(false)}
                className="btn btn-secondary"
              >
                Cancel
              </button>
            </div>
          </div>
        </div>
      )}

      {loading && topics.length === 0 ? (
        <div className="text-center py-8">
          <Loader2 className="w-6 h-6 animate-spin text-gray-400 mx-auto mb-2" />
          <p className="text-sm text-gray-600 dark:text-gray-400">Loading topics...</p>
        </div>
      ) : topics.length === 0 ? (
        <div className="text-center py-12">
          <p className="text-gray-600 dark:text-gray-400">
            No topics yet. Add your first topic to get started!
          </p>
        </div>
      ) : (
        <div className="space-y-3">
          {topics.map((topic) => (
            <div
              key={topic.id}
              className="flex items-center justify-between p-4 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700"
            >
              <div className="flex-1">
                <h4 className="font-medium text-gray-900 dark:text-white mb-1">
                  {topic.name}
                </h4>
                {topic.description && (
                  <p className="text-sm text-gray-600 dark:text-gray-400">
                    {topic.description}
                  </p>
                )}
              </div>
              <div className="flex items-center gap-3 ml-4">
                <label className="relative inline-flex items-center cursor-pointer">
                  <input
                    type="checkbox"
                    checked={topic.enabled}
                    onChange={(e) => handleToggleTopic(topic.id, e.target.checked)}
                    className="sr-only peer"
                  />
                  <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-primary-300 dark:peer-focus:ring-primary-800 rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all dark:border-gray-600 peer-checked:bg-primary-600"></div>
                </label>
                <button
                  onClick={() => handleDeleteTopic(topic.id)}
                  className="p-2 text-gray-400 hover:text-red-600 dark:hover:text-red-400 transition-colors"
                  aria-label="Delete topic"
                >
                  <Trash2 className="w-4 h-4" />
                </button>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

function MCPServersTab() {
  const { servers, loading, toggleServer } = useMCPServers();

  const handleToggle = async (serverId: string, enabled: boolean) => {
    await toggleServer(serverId, enabled);
  };

  return (
    <div>
      <div className="mb-6">
        <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-1">
          MCP Servers
        </h2>
        <p className="text-sm text-gray-600 dark:text-gray-400">
          Enable or disable Model Context Protocol servers for research
        </p>
      </div>

      {loading && servers.length === 0 ? (
        <div className="text-center py-8">
          <Loader2 className="w-6 h-6 animate-spin text-gray-400 mx-auto mb-2" />
          <p className="text-sm text-gray-600 dark:text-gray-400">Loading servers...</p>
        </div>
      ) : servers.length === 0 ? (
        <div className="text-center py-12">
          <p className="text-gray-600 dark:text-gray-400">
            No MCP servers configured. Add servers to your config file.
          </p>
        </div>
      ) : (
        <div className="space-y-3">
          {servers.map((server) => (
            <div
              key={server.id}
              className="flex items-center justify-between p-4 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700"
            >
              <div className="flex-1">
                <div className="flex items-center gap-2 mb-1">
                  <h4 className="font-medium text-gray-900 dark:text-white">
                    {server.name}
                  </h4>
                  {server.enabled && (
                    <CheckCircle2 className="w-4 h-4 text-green-600 dark:text-green-400" />
                  )}
                </div>
                {server.last_used && (
                  <p className="text-xs text-gray-500 dark:text-gray-400">
                    Last used: {new Date(server.last_used).toLocaleDateString()}
                  </p>
                )}
              </div>
              <label className="relative inline-flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  checked={server.enabled}
                  onChange={(e) => handleToggle(server.id, e.target.checked)}
                  className="sr-only peer"
                />
                <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-primary-300 dark:peer-focus:ring-primary-800 rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all dark:border-gray-600 peer-checked:bg-primary-600"></div>
              </label>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

function ResearchSettingsTab() {
  const { settings, loading, updateSettings, runResearch } = useSettings();
  const [localSettings, setLocalSettings] = useState(settings);
  const [saving, setSaving] = useState(false);
  const [running, setRunning] = useState(false);

  const handleSave = async () => {
    if (!localSettings) return;

    setSaving(true);
    try {
      await updateSettings(localSettings);
    } finally {
      setSaving(false);
    }
  };

  const handleRunNow = async () => {
    setRunning(true);
    try {
      await runResearch();
      alert('Research started! Check back in a few minutes for new briefings.');
    } catch (err) {
      alert('Failed to start research: ' + (err instanceof Error ? err.message : 'Unknown error'));
    } finally {
      setRunning(false);
    }
  };

  if (loading || !settings) {
    return (
      <div className="text-center py-8">
        <Loader2 className="w-6 h-6 animate-spin text-gray-400 mx-auto mb-2" />
        <p className="text-sm text-gray-600 dark:text-gray-400">Loading settings...</p>
      </div>
    );
  }

  return (
    <div>
      <div className="mb-6">
        <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-1">
          Research Configuration
        </h2>
        <p className="text-sm text-gray-600 dark:text-gray-400">
          Configure how and when research is performed
        </p>
      </div>

      <div className="space-y-6">
        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            Schedule (Cron Expression)
          </label>
          <input
            type="text"
            value={localSettings?.schedule_cron || settings.schedule_cron}
            onChange={(e) => setLocalSettings({ ...settings, schedule_cron: e.target.value })}
            placeholder="0 8 * * *"
            className="input w-full"
          />
          <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
            When to run automatic research (e.g., "0 8 * * *" for daily at 8 AM)
          </p>
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            AI Model
          </label>
          <select
            value={localSettings?.model || settings.model}
            onChange={(e) => setLocalSettings({ ...settings, model: e.target.value })}
            className="input w-full"
          >
            <option value="claude-3-5-sonnet-20241022">Claude 3.5 Sonnet</option>
            <option value="claude-3-opus-20240229">Claude 3 Opus</option>
            <option value="claude-3-haiku-20240307">Claude 3 Haiku</option>
          </select>
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            Research Depth
          </label>
          <select
            value={localSettings?.research_depth || settings.research_depth}
            onChange={(e) => setLocalSettings({ ...settings, research_depth: e.target.value as 'shallow' | 'medium' | 'deep' })}
            className="input w-full"
          >
            <option value="shallow">Shallow (faster, fewer sources)</option>
            <option value="medium">Medium (balanced)</option>
            <option value="deep">Deep (thorough, slower)</option>
          </select>
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            Max Sources Per Topic
          </label>
          <input
            type="number"
            min="1"
            max="50"
            value={localSettings?.max_sources_per_topic || settings.max_sources_per_topic}
            onChange={(e) => setLocalSettings({ ...settings, max_sources_per_topic: parseInt(e.target.value) })}
            className="input w-full"
          />
        </div>

        <div className="flex items-center gap-3">
          <label className="relative inline-flex items-center cursor-pointer">
            <input
              type="checkbox"
              checked={localSettings?.enable_notifications ?? settings.enable_notifications}
              onChange={(e) => setLocalSettings({ ...settings, enable_notifications: e.target.checked })}
              className="sr-only peer"
            />
            <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-primary-300 dark:peer-focus:ring-primary-800 rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all dark:border-gray-600 peer-checked:bg-primary-600"></div>
          </label>
          <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
            Enable Notifications
          </span>
        </div>

        <div className="pt-6 border-t border-gray-200 dark:border-gray-700 flex items-center justify-between">
          <button
            onClick={handleRunNow}
            disabled={running}
            className="btn btn-secondary flex items-center gap-2"
          >
            {running ? (
              <Loader2 className="w-4 h-4 animate-spin" />
            ) : (
              <Play className="w-4 h-4" />
            )}
            Run Research Now
          </button>

          <button
            onClick={handleSave}
            disabled={saving}
            className="btn btn-primary flex items-center gap-2"
          >
            {saving ? (
              <Loader2 className="w-4 h-4 animate-spin" />
            ) : (
              <Save className="w-4 h-4" />
            )}
            Save Settings
          </button>
        </div>
      </div>
    </div>
  );
}
