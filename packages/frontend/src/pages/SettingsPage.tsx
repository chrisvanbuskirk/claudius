import { useState } from 'react';
import { Save, Plus, X, Trash2, CheckCircle2, Loader2, Play, Key, Eye, EyeOff } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { useTopics, useMCPServers, useSettings, useApiKey } from '../hooks/useTauri';

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

// Preset MCP server configurations
const MCP_PRESETS = [
  { name: 'Filesystem', command: 'npx -y @modelcontextprotocol/server-filesystem', description: 'Access local files and directories' },
  { name: 'GitHub', command: 'npx -y @modelcontextprotocol/server-github', description: 'Access GitHub repositories and issues' },
  { name: 'Brave Search', command: 'npx -y @anthropic/server-brave-search', description: 'Web search via Brave' },
  { name: 'Memory', command: 'npx -y @modelcontextprotocol/server-memory', description: 'Persistent memory storage' },
  { name: 'Fetch', command: 'npx -y @anthropic/server-fetch', description: 'Fetch web content' },
];

function MCPServersTab() {
  const { servers, loading, addServer, toggleServer, removeServer } = useMCPServers();
  const [showAddForm, setShowAddForm] = useState(false);
  const [newServerName, setNewServerName] = useState('');
  const [newServerCommand, setNewServerCommand] = useState('');
  const [saving, setSaving] = useState(false);

  const handleToggle = async (serverId: string, enabled: boolean) => {
    await toggleServer(serverId, enabled);
  };

  const handleAddServer = async () => {
    if (!newServerName.trim() || !newServerCommand.trim()) return;

    setSaving(true);
    try {
      await addServer(newServerName, { command: newServerCommand });
      setNewServerName('');
      setNewServerCommand('');
      setShowAddForm(false);
    } finally {
      setSaving(false);
    }
  };

  const handleAddPreset = async (preset: typeof MCP_PRESETS[0]) => {
    setSaving(true);
    try {
      await addServer(preset.name, { command: preset.command });
    } finally {
      setSaving(false);
    }
  };

  const handleRemoveServer = async (serverId: string) => {
    if (confirm('Are you sure you want to remove this MCP server?')) {
      await removeServer(serverId);
    }
  };

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <div>
          <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-1">
            MCP Servers
          </h2>
          <p className="text-sm text-gray-600 dark:text-gray-400">
            Model Context Protocol servers for enhanced research
          </p>
        </div>
        {!showAddForm && (
          <button
            onClick={() => setShowAddForm(true)}
            className="btn btn-primary flex items-center gap-2"
          >
            <Plus className="w-4 h-4" />
            Add Server
          </button>
        )}
      </div>

      {showAddForm && (
        <div className="mb-6 p-4 bg-gray-50 dark:bg-gray-800/50 rounded-lg border border-gray-200 dark:border-gray-700">
          <div className="flex items-start justify-between mb-4">
            <h3 className="font-medium text-gray-900 dark:text-white">Add MCP Server</h3>
            <button
              onClick={() => setShowAddForm(false)}
              className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
            >
              <X className="w-5 h-5" />
            </button>
          </div>

          {/* Quick add presets */}
          <div className="mb-4">
            <p className="text-sm text-gray-600 dark:text-gray-400 mb-2">Quick add:</p>
            <div className="flex flex-wrap gap-2">
              {MCP_PRESETS.filter(p => !servers.some(s => s.name === p.name)).map((preset) => (
                <button
                  key={preset.name}
                  onClick={() => handleAddPreset(preset)}
                  disabled={saving}
                  className="px-3 py-1.5 text-sm bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-600 transition-colors"
                  title={preset.description}
                >
                  + {preset.name}
                </button>
              ))}
            </div>
          </div>

          <div className="border-t border-gray-200 dark:border-gray-700 pt-4">
            <p className="text-sm text-gray-600 dark:text-gray-400 mb-3">Or add custom:</p>
            <div className="space-y-3">
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                  Server Name
                </label>
                <input
                  type="text"
                  value={newServerName}
                  onChange={(e) => setNewServerName(e.target.value)}
                  placeholder="e.g., My Custom Server"
                  className="input w-full"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                  Command
                </label>
                <input
                  type="text"
                  value={newServerCommand}
                  onChange={(e) => setNewServerCommand(e.target.value)}
                  placeholder="e.g., npx -y @modelcontextprotocol/server-name"
                  className="input w-full font-mono text-sm"
                />
              </div>
              <div className="flex gap-2">
                <button
                  onClick={handleAddServer}
                  disabled={!newServerName.trim() || !newServerCommand.trim() || saving}
                  className="btn btn-primary flex items-center gap-2"
                >
                  {saving ? (
                    <Loader2 className="w-4 h-4 animate-spin" />
                  ) : (
                    <Plus className="w-4 h-4" />
                  )}
                  Add Server
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
        </div>
      )}

      {loading && servers.length === 0 ? (
        <div className="text-center py-8">
          <Loader2 className="w-6 h-6 animate-spin text-gray-400 mx-auto mb-2" />
          <p className="text-sm text-gray-600 dark:text-gray-400">Loading servers...</p>
        </div>
      ) : servers.length === 0 ? (
        <div className="text-center py-12">
          <p className="text-gray-600 dark:text-gray-400 mb-4">
            No MCP servers configured yet.
          </p>
          <button
            onClick={() => setShowAddForm(true)}
            className="btn btn-primary"
          >
            Add Your First Server
          </button>
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
                {typeof server.config?.command === 'string' && (
                  <p className="text-xs text-gray-500 dark:text-gray-400 font-mono truncate max-w-md">
                    {String(server.config.command)}
                  </p>
                )}
                {server.last_used && (
                  <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                    Last used: {new Date(server.last_used).toLocaleDateString()}
                  </p>
                )}
              </div>
              <div className="flex items-center gap-3 ml-4">
                <label className="relative inline-flex items-center cursor-pointer">
                  <input
                    type="checkbox"
                    checked={server.enabled}
                    onChange={(e) => handleToggle(server.id, e.target.checked)}
                    className="sr-only peer"
                  />
                  <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-primary-300 dark:peer-focus:ring-primary-800 rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all dark:border-gray-600 peer-checked:bg-primary-600"></div>
                </label>
                <button
                  onClick={() => handleRemoveServer(server.id)}
                  className="p-2 text-gray-400 hover:text-red-600 dark:hover:text-red-400 transition-colors"
                  aria-label="Remove server"
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

function ResearchSettingsTab() {
  const { settings, loading, updateSettings, runResearch } = useSettings();
  const { maskedKey, hasKey, loading: apiKeyLoading, setApiKey } = useApiKey();
  const [localSettings, setLocalSettings] = useState(settings);

  // Sync localSettings when settings load from backend
  const settingsLoaded = !loading && settings;
  if (settingsLoaded && !localSettings) {
    setLocalSettings(settings);
  }
  const [saving, setSaving] = useState(false);
  const [running, setRunning] = useState(false);

  // API Key state
  const [newApiKey, setNewApiKey] = useState('');
  const [showApiKey, setShowApiKey] = useState(false);
  const [savingApiKey, setSavingApiKey] = useState(false);
  const [apiKeyError, setApiKeyError] = useState<string | null>(null);

  const handleSaveApiKey = async () => {
    if (!newApiKey.trim()) return;

    if (!newApiKey.startsWith('sk-ant-')) {
      setApiKeyError("Invalid API key format. Anthropic API keys start with 'sk-ant-'");
      return;
    }

    setSavingApiKey(true);
    setApiKeyError(null);
    try {
      const success = await setApiKey(newApiKey);
      if (success) {
        setNewApiKey('');
        setShowApiKey(false);
      }
    } catch (err) {
      setApiKeyError(err instanceof Error ? err.message : 'Failed to save API key');
    } finally {
      setSavingApiKey(false);
    }
  };

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
        {/* API Key Section */}
        <div className="p-4 bg-gray-50 dark:bg-gray-800/50 rounded-lg border border-gray-200 dark:border-gray-700">
          <div className="flex items-center gap-2 mb-3">
            <Key className="w-5 h-5 text-gray-600 dark:text-gray-400" />
            <h3 className="font-medium text-gray-900 dark:text-white">Anthropic API Key</h3>
          </div>

          {apiKeyLoading ? (
            <div className="flex items-center gap-2 text-sm text-gray-500">
              <Loader2 className="w-4 h-4 animate-spin" />
              Checking API key...
            </div>
          ) : hasKey ? (
            <div className="space-y-3">
              <div className="flex items-center gap-3 p-3 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg">
                <CheckCircle2 className="w-5 h-5 text-green-600 dark:text-green-400 flex-shrink-0" />
                <div className="flex-1">
                  <p className="text-sm font-medium text-green-800 dark:text-green-300">API Key Configured</p>
                  <p className="text-sm text-green-600 dark:text-green-400 tracking-wider">{maskedKey}</p>
                </div>
              </div>
              <div className="pt-2">
                <p className="text-xs text-gray-500 dark:text-gray-400 mb-2">Replace with a different key:</p>
                <div className="flex gap-2">
                  <div className="relative flex-1">
                    <input
                      type={showApiKey ? 'text' : 'password'}
                      value={newApiKey}
                      onChange={(e) => {
                        setNewApiKey(e.target.value);
                        setApiKeyError(null);
                      }}
                      placeholder="sk-ant-..."
                      className="input w-full pr-10 font-mono text-sm"
                    />
                    <button
                      type="button"
                      onClick={() => setShowApiKey(!showApiKey)}
                      className="absolute right-2 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                    >
                      {showApiKey ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                    </button>
                  </div>
                  <button
                    onClick={handleSaveApiKey}
                    disabled={!newApiKey.trim() || savingApiKey}
                    className="btn btn-primary flex items-center gap-1.5"
                  >
                    {savingApiKey ? <Loader2 className="w-4 h-4 animate-spin" /> : <Save className="w-4 h-4" />}
                    Save
                  </button>
                </div>
                {apiKeyError && (
                  <p className="text-xs text-red-600 dark:text-red-400 mt-2">{apiKeyError}</p>
                )}
              </div>
            </div>
          ) : (
            <div className="space-y-3">
              <p className="text-sm text-yellow-600 dark:text-yellow-400">
                No API key configured. Add your Anthropic API key to enable research.
              </p>
              <div className="flex gap-2">
                <div className="relative flex-1">
                  <input
                    type={showApiKey ? 'text' : 'password'}
                    value={newApiKey}
                    onChange={(e) => {
                      setNewApiKey(e.target.value);
                      setApiKeyError(null);
                    }}
                    placeholder="sk-ant-..."
                    className="input w-full pr-10 font-mono text-sm"
                  />
                  <button
                    type="button"
                    onClick={() => setShowApiKey(!showApiKey)}
                    className="absolute right-2 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                  >
                    {showApiKey ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                  </button>
                </div>
                <button
                  onClick={handleSaveApiKey}
                  disabled={!newApiKey.trim() || savingApiKey}
                  className="btn btn-primary flex items-center gap-1.5"
                >
                  {savingApiKey ? <Loader2 className="w-4 h-4 animate-spin" /> : <Save className="w-4 h-4" />}
                  Save
                </button>
              </div>
              {apiKeyError && (
                <p className="text-xs text-red-600 dark:text-red-400 mt-2">{apiKeyError}</p>
              )}
              <p className="text-xs text-gray-500 dark:text-gray-400">
                Get your API key from <a href="https://console.anthropic.com" target="_blank" rel="noopener noreferrer" className="text-primary-600 hover:underline">console.anthropic.com</a>
              </p>
            </div>
          )}
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            Research Schedule
          </label>
          <select
            value={localSettings?.schedule_cron || settings.schedule_cron}
            onChange={(e) => setLocalSettings(prev => ({ ...prev!, schedule_cron: e.target.value }))}
            className="input w-full"
          >
            <option value="0 6 * * *">Daily at 6:00 AM</option>
            <option value="0 7 * * *">Daily at 7:00 AM</option>
            <option value="0 8 * * *">Daily at 8:00 AM</option>
            <option value="0 9 * * *">Daily at 9:00 AM</option>
            <option value="0 12 * * *">Daily at 12:00 PM</option>
            <option value="0 18 * * *">Daily at 6:00 PM</option>
            <option value="0 8 * * 1-5">Weekdays at 8:00 AM</option>
            <option value="0 9 * * 1-5">Weekdays at 9:00 AM</option>
            <option value="0 8,18 * * *">Twice daily (8 AM & 6 PM)</option>
            <option value="0 0 * * 0">Weekly on Sunday</option>
            <option value="0 0 * * 1">Weekly on Monday</option>
          </select>
          <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
            When to automatically run research for your topics
          </p>
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            AI Model
          </label>
          <select
            value={localSettings?.model || settings.model}
            onChange={(e) => setLocalSettings(prev => ({ ...prev!, model: e.target.value }))}
            className="input w-full"
          >
            <option value="claude-haiku-4-5-20241022">Claude Haiku 4.5 (fastest, cheapest)</option>
            <option value="claude-sonnet-4-5-20250929">Claude Sonnet 4.5 (balanced)</option>
            <option value="claude-opus-4-5-20251101">Claude Opus 4.5 (most capable)</option>
          </select>
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            Research Depth
          </label>
          <select
            value={localSettings?.research_depth || settings.research_depth}
            onChange={(e) => setLocalSettings(prev => ({ ...prev!, research_depth: e.target.value as 'shallow' | 'medium' | 'deep' }))}
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
            onChange={(e) => setLocalSettings(prev => ({ ...prev!, max_sources_per_topic: parseInt(e.target.value) }))}
            className="input w-full"
          />
        </div>

        <div className="flex items-center gap-3">
          <label className="relative inline-flex items-center cursor-pointer">
            <input
              type="checkbox"
              checked={localSettings?.enable_notifications ?? settings.enable_notifications}
              onChange={(e) => setLocalSettings(prev => ({ ...prev!, enable_notifications: e.target.checked }))}
              className="sr-only peer"
            />
            <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-primary-300 dark:peer-focus:ring-primary-800 rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all dark:border-gray-600 peer-checked:bg-primary-600"></div>
          </label>
          <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
            Enable Notifications
          </span>
        </div>

        <div className="flex items-center gap-3">
          <label className="relative inline-flex items-center cursor-pointer">
            <input
              type="checkbox"
              checked={localSettings?.notification_sound ?? settings.notification_sound ?? true}
              onChange={(e) => setLocalSettings(prev => ({ ...prev!, notification_sound: e.target.checked }))}
              disabled={!(localSettings?.enable_notifications ?? settings.enable_notifications)}
              className="sr-only peer"
            />
            <div className={`w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-primary-300 dark:peer-focus:ring-primary-800 rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all dark:border-gray-600 peer-checked:bg-primary-600 ${!(localSettings?.enable_notifications ?? settings.enable_notifications) ? 'opacity-50' : ''}`}></div>
          </label>
          <span className={`text-sm font-medium text-gray-700 dark:text-gray-300 ${!(localSettings?.enable_notifications ?? settings.enable_notifications) ? 'opacity-50' : ''}`}>
            Notification Sound
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
