import { useState } from 'react';
import { Plus, X, Trash2, CheckCircle2, Loader2, Play, Key, Eye, EyeOff, Edit2, AlertTriangle, Globe, Save } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { useTopics, useMCPServers, useSettings, useApiKey } from '../hooks/useTauri';
import { MagneticButton } from '../components/MagneticButton';
import { motion, AnimatePresence } from 'framer-motion';

type Tab = 'interests' | 'mcp' | 'research';

// Confirmation Dialog Component
interface ConfirmDialogProps {
  isOpen: boolean;
  title: string;
  message: string;
  confirmLabel?: string;
  cancelLabel?: string;
  onConfirm: () => void;
  onCancel: () => void;
}

function ConfirmDialog({
  isOpen,
  title,
  message,
  confirmLabel = 'Delete',
  cancelLabel = 'Cancel',
  onConfirm,
  onCancel,
}: ConfirmDialogProps) {
  if (!isOpen) return null;

  return (
    <AnimatePresence>
      <motion.div
        className="fixed inset-0 z-50 flex items-center justify-center"
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
      >
        {/* Backdrop */}
        <div
          className="absolute inset-0 bg-black/50 backdrop-blur-sm"
          onClick={onCancel}
        />
        {/* Dialog */}
        <motion.div
          className="relative z-10 bg-white dark:bg-gray-800 rounded-xl shadow-2xl p-6 max-w-md w-full mx-4 border border-gray-200 dark:border-gray-700"
          initial={{ scale: 0.95, opacity: 0 }}
          animate={{ scale: 1, opacity: 1 }}
          exit={{ scale: 0.95, opacity: 0 }}
        >
          <div className="flex items-start gap-4">
            <div className="p-2 bg-red-100 dark:bg-red-900/30 rounded-full">
              <AlertTriangle className="w-6 h-6 text-red-600 dark:text-red-400" />
            </div>
            <div className="flex-1">
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">
                {title}
              </h3>
              <p className="text-sm text-gray-600 dark:text-gray-400 mb-4">
                {message}
              </p>
              <div className="flex gap-3 justify-end">
                <button
                  onClick={onCancel}
                  className="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 rounded-lg transition-colors"
                >
                  {cancelLabel}
                </button>
                <button
                  onClick={onConfirm}
                  className="px-4 py-2 text-sm font-medium text-white bg-red-600 hover:bg-red-700 rounded-lg transition-colors"
                >
                  {confirmLabel}
                </button>
              </div>
            </div>
          </div>
        </motion.div>
      </motion.div>
    </AnimatePresence>
  );
}

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

      <div className="glass-card mb-6">
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
  const [deleteConfirm, setDeleteConfirm] = useState<{ id: string; name: string } | null>(null);

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

  const handleDeleteTopic = (topic: { id: string; name: string }) => {
    setDeleteConfirm(topic);
  };

  const confirmDeleteTopic = async () => {
    if (deleteConfirm) {
      await deleteTopic(deleteConfirm.id);
      setDeleteConfirm(null);
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
          <MagneticButton
            onClick={() => setShowAddForm(true)}
            variant="primary"
            className="flex items-center gap-2"
          >
            <Plus className="w-4 h-4" />
            Add Topic
          </MagneticButton>
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
              <MagneticButton
                onClick={handleAddTopic}
                disabled={!newTopicName.trim() || saving}
                variant="primary"
                className="flex items-center gap-2"
              >
                {saving ? (
                  <Loader2 className="w-4 h-4 animate-spin" />
                ) : (
                  <Plus className="w-4 h-4" />
                )}
                Add Topic
              </MagneticButton>
              <MagneticButton
                onClick={() => setShowAddForm(false)}
                variant="secondary"
              >
                Cancel
              </MagneticButton>
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
              className="flex items-center justify-between p-4 glass-nav rounded-lg"
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
                  onClick={() => handleDeleteTopic({ id: topic.id, name: topic.name })}
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

      {/* Delete Confirmation Dialog */}
      <ConfirmDialog
        isOpen={deleteConfirm !== null}
        title="Delete Topic"
        message={`Are you sure you want to delete "${deleteConfirm?.name}"? This action cannot be undone.`}
        confirmLabel="Delete"
        onConfirm={confirmDeleteTopic}
        onCancel={() => setDeleteConfirm(null)}
      />
    </div>
  );
}

// Preset MCP server configurations
const MCP_PRESETS = [
  { name: 'Filesystem', command: 'npx -y @modelcontextprotocol/server-filesystem', description: 'Access local files and directories' },
  { name: 'GitHub', command: 'npx -y @modelcontextprotocol/server-github', description: 'Access GitHub repositories and issues', env: { 'GITHUB_TOKEN': '' } },
  { name: 'Brave Search', command: 'npx -y @anthropic/server-brave-search', description: 'Web search via Brave', env: { 'BRAVE_API_KEY': '' } },
  { name: 'Memory', command: 'npx -y @modelcontextprotocol/server-memory', description: 'Persistent memory storage' },
  { name: 'Fetch', command: 'npx -y @anthropic/server-fetch', description: 'Fetch web content' },
];

function MCPServersTab() {
  const { servers, loading, addServer, updateServer, toggleServer, removeServer } = useMCPServers();
  const [showAddForm, setShowAddForm] = useState(false);
  const [newServerName, setNewServerName] = useState('');
  const [newServerCommand, setNewServerCommand] = useState('');
  const [newServerArgs, setNewServerArgs] = useState<string[]>([]);
  const [newServerEnv, setNewServerEnv] = useState<Record<string, string>>({});
  const [showNewEnvValues, setShowNewEnvValues] = useState<Record<string, boolean>>({});
  const [editingServerId, setEditingServerId] = useState<string | null>(null);
  const [editServerName, setEditServerName] = useState('');
  const [editServerCommand, setEditServerCommand] = useState('');
  const [editServerArgs, setEditServerArgs] = useState<string[]>([]);
  const [editServerEnv, setEditServerEnv] = useState<Record<string, string>>({});
  const [editServerEnabled, setEditServerEnabled] = useState(true);
  const [showEnvValues, setShowEnvValues] = useState<Record<string, boolean>>({});
  const [saving, setSaving] = useState(false);
  const [deleteConfirm, setDeleteConfirm] = useState<{ id: string; name: string } | null>(null);

  const handleToggle = async (serverId: string, enabled: boolean) => {
    await toggleServer(serverId, enabled);
  };

  const handleAddServer = async () => {
    if (!newServerName.trim() || !newServerCommand.trim()) return;

    setSaving(true);
    try {
      const config: Record<string, unknown> = {
        command: newServerCommand.trim(),
      };

      if (newServerArgs.length > 0) {
        config.args = newServerArgs;
      }

      if (Object.keys(newServerEnv).length > 0) {
        config.env = newServerEnv;
      }

      await addServer(newServerName, config);
      setNewServerName('');
      setNewServerCommand('');
      setNewServerArgs([]);
      setNewServerEnv({});
      setShowNewEnvValues({});
      setShowAddForm(false);
    } finally {
      setSaving(false);
    }
  };

  const handleAddPreset = async (preset: typeof MCP_PRESETS[0]) => {
    setSaving(true);
    try {
      // Parse preset command string into command + args
      const parts = preset.command.trim().split(/\s+/);
      const command = parts[0] || 'npx';
      const args = parts.slice(1);

      const config: Record<string, unknown> = { command };
      if (args.length > 0) {
        config.args = args;
      }
      if (preset.env && Object.keys(preset.env).length > 0) {
        config.env = preset.env;
      }

      await addServer(preset.name, config);
    } finally {
      setSaving(false);
    }
  };

  const handleEditServer = (server: typeof servers[0]) => {
    setEditingServerId(server.id);
    setEditServerName(server.name);
    setEditServerEnabled(server.enabled);

    // Extract command, args, and env separately
    let command = 'npx';
    let args: string[] = [];
    let env: Record<string, string> = {};

    if (server.config) {
      if (typeof server.config === 'string') {
        command = server.config;
      } else {
        if (server.config.command) {
          command = String(server.config.command);
        }
        if (Array.isArray(server.config.args)) {
          args = server.config.args.map(String);
        }
        if (server.config.env && typeof server.config.env === 'object') {
          env = server.config.env as Record<string, string>;
        }
      }
    }

    setEditServerCommand(command);
    setEditServerArgs(args);
    setEditServerEnv(env);
    setShowEnvValues({});
    setShowAddForm(false);
  };

  const handleUpdateServer = async () => {
    if (!editingServerId || !editServerName.trim() || !editServerCommand.trim()) return;

    setSaving(true);
    try {
      const config: Record<string, unknown> = {
        command: editServerCommand.trim(),
      };

      if (editServerArgs.length > 0) {
        config.args = editServerArgs;
      }

      if (Object.keys(editServerEnv).length > 0) {
        config.env = editServerEnv;
      }

      await updateServer(editingServerId, editServerName, config);

      // Also update enabled state if changed
      const server = servers.find(s => s.id === editingServerId);
      if (server && server.enabled !== editServerEnabled) {
        await toggleServer(editingServerId, editServerEnabled);
      }

      setEditingServerId(null);
      setEditServerName('');
      setEditServerCommand('');
      setEditServerArgs([]);
      setEditServerEnv({});
      setEditServerEnabled(true);
      setShowEnvValues({});
    } finally {
      setSaving(false);
    }
  };

  const handleCancelEdit = () => {
    setEditingServerId(null);
    setEditServerName('');
    setEditServerCommand('');
    setEditServerArgs([]);
    setEditServerEnv({});
    setEditServerEnabled(true);
    setShowEnvValues({});
  };

  const handleRemoveServer = (server: { id: string; name: string }) => {
    setDeleteConfirm(server);
  };

  const confirmRemoveServer = async () => {
    if (deleteConfirm) {
      await removeServer(deleteConfirm.id);
      setDeleteConfirm(null);
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
          <MagneticButton
            onClick={() => setShowAddForm(true)}
            variant="primary"
            className="flex items-center gap-2"
          >
            <Plus className="w-4 h-4" />
            Add Server
          </MagneticButton>
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
              {/* Command */}
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                  Command
                </label>
                <input
                  type="text"
                  value={newServerCommand}
                  onChange={(e) => setNewServerCommand(e.target.value)}
                  placeholder="npx"
                  className="input w-full font-mono text-sm"
                />
              </div>

              {/* Arguments */}
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                  Arguments
                </label>
                <div className="space-y-2">
                  {newServerArgs.map((arg, idx) => (
                    <div key={idx} className="flex gap-2">
                      <input
                        type="text"
                        value={arg}
                        onChange={(e) => {
                          const newArgs = [...newServerArgs];
                          newArgs[idx] = e.target.value;
                          setNewServerArgs(newArgs);
                        }}
                        className="input flex-1 font-mono text-sm"
                        placeholder="argument"
                      />
                      <button
                        onClick={() => setNewServerArgs(newServerArgs.filter((_, i) => i !== idx))}
                        className="p-2 text-gray-400 hover:text-red-600 dark:hover:text-red-400"
                      >
                        <Trash2 className="w-4 h-4" />
                      </button>
                    </div>
                  ))}
                  <button
                    onClick={() => setNewServerArgs([...newServerArgs, ''])}
                    className="text-sm text-primary-600 dark:text-primary-400 hover:text-primary-700 dark:hover:text-primary-300"
                  >
                    + Add Argument
                  </button>
                </div>
              </div>

              {/* Environment Variables */}
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                  Environment Variables
                </label>
                <div className="space-y-2">
                  {Object.entries(newServerEnv).map(([key, value]) => (
                    <div key={key} className="flex gap-2">
                      <input
                        type="text"
                        value={key}
                        onChange={(e) => {
                          const newEnv = { ...newServerEnv };
                          delete newEnv[key];
                          newEnv[e.target.value] = value;
                          setNewServerEnv(newEnv);
                        }}
                        className="input flex-1 font-mono text-sm"
                        placeholder="KEY"
                      />
                      <div className="flex-1 relative">
                        <input
                          type={showNewEnvValues[key] ? 'text' : 'password'}
                          value={value}
                          onChange={(e) => {
                            setNewServerEnv({ ...newServerEnv, [key]: e.target.value });
                          }}
                          className="input w-full font-mono text-sm pr-10"
                          placeholder="value"
                        />
                        <button
                          onClick={() => setShowNewEnvValues({ ...showNewEnvValues, [key]: !showNewEnvValues[key] })}
                          className="absolute right-2 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                        >
                          {showNewEnvValues[key] ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                        </button>
                      </div>
                      <button
                        onClick={() => {
                          const newEnv = { ...newServerEnv };
                          delete newEnv[key];
                          setNewServerEnv(newEnv);
                        }}
                        className="p-2 text-gray-400 hover:text-red-600 dark:hover:text-red-400"
                      >
                        <Trash2 className="w-4 h-4" />
                      </button>
                    </div>
                  ))}
                  <button
                    onClick={() => {
                      const newKey = `VAR_${Object.keys(newServerEnv).length + 1}`;
                      setNewServerEnv({ ...newServerEnv, [newKey]: '' });
                    }}
                    className="text-sm text-primary-600 dark:text-primary-400 hover:text-primary-700 dark:hover:text-primary-300"
                  >
                    + Add Variable
                  </button>
                </div>
              </div>

              <div className="flex gap-2 pt-2">
                <MagneticButton
                  onClick={handleAddServer}
                  disabled={!newServerName.trim() || !newServerCommand.trim() || saving}
                  variant="primary"
                  className="flex items-center gap-2"
                >
                  {saving ? (
                    <Loader2 className="w-4 h-4 animate-spin" />
                  ) : (
                    <Plus className="w-4 h-4" />
                  )}
                  Add Server
                </MagneticButton>
                <MagneticButton
                  onClick={() => setShowAddForm(false)}
                  variant="secondary"
                >
                  Cancel
                </MagneticButton>
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
          <MagneticButton
            onClick={() => setShowAddForm(true)}
            variant="primary"
          >
            Add Your First Server
          </MagneticButton>
        </div>
      ) : (
        <div className="space-y-3">
          {servers.map((server) => (
            editingServerId === server.id ? (
              <div key={server.id} className="p-4 bg-gray-50 dark:bg-gray-800/50 rounded-lg border border-gray-200 dark:border-gray-700">
                <div className="flex items-start justify-between mb-4">
                  <h3 className="font-medium text-gray-900 dark:text-white">Edit Server</h3>
                  <button
                    onClick={handleCancelEdit}
                    className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                  >
                    <X className="w-5 h-5" />
                  </button>
                </div>
                <div className="space-y-4">
                  {/* Server Name */}
                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                      Server Name
                    </label>
                    <input
                      type="text"
                      value={editServerName}
                      onChange={(e) => setEditServerName(e.target.value)}
                      className="input w-full"
                      placeholder="e.g., Brave Search"
                    />
                  </div>

                  {/* Command */}
                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                      Command
                    </label>
                    <input
                      type="text"
                      value={editServerCommand}
                      onChange={(e) => setEditServerCommand(e.target.value)}
                      className="input w-full font-mono text-sm"
                      placeholder="npx"
                    />
                  </div>

                  {/* Arguments */}
                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                      Arguments
                    </label>
                    <div className="space-y-2">
                      {editServerArgs.map((arg, idx) => (
                        <div key={idx} className="flex gap-2">
                          <input
                            type="text"
                            value={arg}
                            onChange={(e) => {
                              const newArgs = [...editServerArgs];
                              newArgs[idx] = e.target.value;
                              setEditServerArgs(newArgs);
                            }}
                            className="input flex-1 font-mono text-sm"
                          />
                          <button
                            onClick={() => setEditServerArgs(editServerArgs.filter((_, i) => i !== idx))}
                            className="p-2 text-gray-400 hover:text-red-600 dark:hover:text-red-400"
                          >
                            <Trash2 className="w-4 h-4" />
                          </button>
                        </div>
                      ))}
                      <button
                        onClick={() => setEditServerArgs([...editServerArgs, ''])}
                        className="text-sm text-primary-600 dark:text-primary-400 hover:text-primary-700 dark:hover:text-primary-300"
                      >
                        + Add Argument
                      </button>
                    </div>
                  </div>

                  {/* Environment Variables */}
                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                      Environment Variables
                    </label>
                    <div className="space-y-2">
                      {Object.entries(editServerEnv).map(([key, value]) => (
                        <div key={key} className="flex gap-2">
                          <input
                            type="text"
                            value={key}
                            onChange={(e) => {
                              const newEnv = { ...editServerEnv };
                              delete newEnv[key];
                              newEnv[e.target.value] = value;
                              setEditServerEnv(newEnv);
                            }}
                            className="input flex-1 font-mono text-sm"
                            placeholder="KEY"
                          />
                          <div className="flex-1 relative">
                            <input
                              type={showEnvValues[key] ? 'text' : 'password'}
                              value={value}
                              onChange={(e) => {
                                setEditServerEnv({ ...editServerEnv, [key]: e.target.value });
                              }}
                              className="input w-full font-mono text-sm pr-10"
                              placeholder="value"
                            />
                            <button
                              onClick={() => setShowEnvValues({ ...showEnvValues, [key]: !showEnvValues[key] })}
                              className="absolute right-2 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                            >
                              {showEnvValues[key] ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                            </button>
                          </div>
                          <button
                            onClick={() => {
                              const newEnv = { ...editServerEnv };
                              delete newEnv[key];
                              setEditServerEnv(newEnv);
                            }}
                            className="p-2 text-gray-400 hover:text-red-600 dark:hover:text-red-400"
                          >
                            <Trash2 className="w-4 h-4" />
                          </button>
                        </div>
                      ))}
                      <button
                        onClick={() => {
                          const newKey = `VAR_${Object.keys(editServerEnv).length + 1}`;
                          setEditServerEnv({ ...editServerEnv, [newKey]: '' });
                        }}
                        className="text-sm text-primary-600 dark:text-primary-400 hover:text-primary-700 dark:hover:text-primary-300"
                      >
                        + Add Variable
                      </button>
                    </div>
                  </div>

                  {/* Enabled Toggle */}
                  <div className="flex items-center gap-2">
                    <input
                      type="checkbox"
                      id="edit-server-enabled"
                      checked={editServerEnabled}
                      onChange={(e) => setEditServerEnabled(e.target.checked)}
                      className="rounded border-gray-300 dark:border-gray-600"
                    />
                    <label htmlFor="edit-server-enabled" className="text-sm text-gray-700 dark:text-gray-300">
                      Enabled
                    </label>
                  </div>

                  {/* Action Buttons */}
                  <div className="flex gap-2 pt-2">
                    <MagneticButton
                      onClick={handleUpdateServer}
                      disabled={!editServerName.trim() || !editServerCommand.trim() || saving}
                      variant="primary"
                      className="flex items-center gap-2"
                    >
                      {saving ? (
                        <Loader2 className="w-4 h-4 animate-spin" />
                      ) : (
                        <Save className="w-4 h-4" />
                      )}
                      Save Changes
                    </MagneticButton>
                    <MagneticButton
                      onClick={handleCancelEdit}
                      variant="secondary"
                    >
                      Cancel
                    </MagneticButton>
                  </div>
                </div>
              </div>
            ) : (
              <div
                key={server.id}
                className="flex items-center justify-between p-4 glass-nav rounded-lg"
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
                    onClick={() => handleEditServer(server)}
                    className="p-2 text-gray-400 hover:text-primary-600 dark:hover:text-primary-400 transition-colors"
                    aria-label="Edit server"
                  >
                    <Edit2 className="w-4 h-4" />
                  </button>
                  <button
                    onClick={() => handleRemoveServer({ id: server.id, name: server.name })}
                    className="p-2 text-gray-400 hover:text-red-600 dark:hover:text-red-400 transition-colors"
                    aria-label="Remove server"
                  >
                    <Trash2 className="w-4 h-4" />
                  </button>
                </div>
              </div>
            )
          ))}
        </div>
      )}

      {/* Delete Confirmation Dialog */}
      <ConfirmDialog
        isOpen={deleteConfirm !== null}
        title="Remove MCP Server"
        message={`Are you sure you want to remove "${deleteConfirm?.name}"? This action cannot be undone.`}
        confirmLabel="Remove"
        onConfirm={confirmRemoveServer}
        onCancel={() => setDeleteConfirm(null)}
      />
    </div>
  );
}

function ResearchSettingsTab() {
  const { settings, loading, updateSettings, runResearch } = useSettings();
  const { maskedKey, hasKey, loading: apiKeyLoading, setApiKey } = useApiKey();
  const [running, setRunning] = useState(false);
  const [savedIndicator, setSavedIndicator] = useState<string | null>(null);

  // Auto-save helper with visual feedback
  const autoSave = async (key: string, value: unknown) => {
    if (!settings) return;
    try {
      await updateSettings({ ...settings, [key]: value });
      setSavedIndicator(key);
      setTimeout(() => setSavedIndicator(null), 1500);
    } catch (err) {
      console.error('Failed to save setting:', err);
    }
  };

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
                  <MagneticButton
                    onClick={handleSaveApiKey}
                    disabled={!newApiKey.trim() || savingApiKey}
                    variant="primary"
                    className="flex items-center gap-1.5"
                  >
                    {savingApiKey ? <Loader2 className="w-4 h-4 animate-spin" /> : <Save className="w-4 h-4" />}
                    Save
                  </MagneticButton>
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
                <MagneticButton
                  onClick={handleSaveApiKey}
                  disabled={!newApiKey.trim() || savingApiKey}
                  variant="primary"
                  className="flex items-center gap-1.5"
                >
                  {savingApiKey ? <Loader2 className="w-4 h-4 animate-spin" /> : <Save className="w-4 h-4" />}
                  Save
                </MagneticButton>
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

        {/* Web Search Section */}
        <div className="p-4 bg-gray-50 dark:bg-gray-800/50 rounded-lg border border-gray-200 dark:border-gray-700">
          <div className="flex items-center gap-2 mb-3">
            <Globe className="w-5 h-5 text-gray-600 dark:text-gray-400" />
            <h3 className="font-medium text-gray-900 dark:text-white">Web Search</h3>
          </div>
          <div className="flex items-start gap-3">
            <label className="relative inline-flex items-center cursor-pointer mt-0.5">
              <input
                type="checkbox"
                checked={settings.enable_web_search ?? false}
                onChange={(e) => autoSave('enable_web_search', e.target.checked)}
                className="sr-only peer"
              />
              <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-primary-300 dark:peer-focus:ring-primary-800 rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all dark:border-gray-600 peer-checked:bg-primary-600"></div>
            </label>
            <div className="flex-1">
              <div className="flex items-center gap-2">
                <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
                  Enable Claude Web Search
                </span>
                {savedIndicator === 'enable_web_search' && (
                  <motion.span
                    initial={{ opacity: 0, scale: 0.8 }}
                    animate={{ opacity: 1, scale: 1 }}
                    exit={{ opacity: 0 }}
                    className="text-xs text-green-600 dark:text-green-400 flex items-center gap-1"
                  >
                    <CheckCircle2 className="w-3 h-3" /> Saved
                  </motion.span>
                )}
              </div>
              <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                Claude's built-in web search for real-time information. <span className="text-amber-600 dark:text-amber-400 font-medium">Costs $0.01 per search</span>.
              </p>
              <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                Alternative: Configure <span className="font-medium">Brave Search</span> MCP server (free tier available) in the MCP Servers tab.
              </p>
            </div>
          </div>
        </div>

        <div>
          <div className="flex items-center gap-2 mb-2">
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
              Research Schedule
            </label>
            {savedIndicator === 'schedule_cron' && (
              <motion.span
                initial={{ opacity: 0, scale: 0.8 }}
                animate={{ opacity: 1, scale: 1 }}
                exit={{ opacity: 0 }}
                className="text-xs text-green-600 dark:text-green-400 flex items-center gap-1"
              >
                <CheckCircle2 className="w-3 h-3" /> Saved
              </motion.span>
            )}
          </div>
          <select
            value={settings.schedule_cron}
            onChange={(e) => autoSave('schedule_cron', e.target.value)}
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
          <div className="flex items-center gap-2 mb-2">
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
              AI Model
            </label>
            {savedIndicator === 'model' && (
              <motion.span
                initial={{ opacity: 0, scale: 0.8 }}
                animate={{ opacity: 1, scale: 1 }}
                exit={{ opacity: 0 }}
                className="text-xs text-green-600 dark:text-green-400 flex items-center gap-1"
              >
                <CheckCircle2 className="w-3 h-3" /> Saved
              </motion.span>
            )}
          </div>
          <select
            value={settings.model}
            onChange={(e) => autoSave('model', e.target.value)}
            className="input w-full"
          >
            <option value="claude-haiku-4-5-20251001">Claude Haiku 4.5 (fastest, cheapest)</option>
            <option value="claude-sonnet-4-5-20250929">Claude Sonnet 4.5 (balanced)</option>
            <option value="claude-opus-4-5-20251101">Claude Opus 4.5 (most capable)</option>
          </select>
        </div>

        <div>
          <div className="flex items-center gap-2 mb-2">
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
              Research Depth
            </label>
            {savedIndicator === 'research_depth' && (
              <motion.span
                initial={{ opacity: 0, scale: 0.8 }}
                animate={{ opacity: 1, scale: 1 }}
                exit={{ opacity: 0 }}
                className="text-xs text-green-600 dark:text-green-400 flex items-center gap-1"
              >
                <CheckCircle2 className="w-3 h-3" /> Saved
              </motion.span>
            )}
          </div>
          <select
            value={settings.research_depth}
            onChange={(e) => autoSave('research_depth', e.target.value)}
            className="input w-full"
          >
            <option value="shallow">Shallow (faster, fewer sources)</option>
            <option value="medium">Medium (balanced)</option>
            <option value="deep">Deep (thorough, slower)</option>
          </select>
        </div>

        <div>
          <div className="flex items-center gap-2 mb-2">
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
              Max Sources Per Topic
            </label>
            {savedIndicator === 'max_sources_per_topic' && (
              <motion.span
                initial={{ opacity: 0, scale: 0.8 }}
                animate={{ opacity: 1, scale: 1 }}
                exit={{ opacity: 0 }}
                className="text-xs text-green-600 dark:text-green-400 flex items-center gap-1"
              >
                <CheckCircle2 className="w-3 h-3" /> Saved
              </motion.span>
            )}
          </div>
          <input
            type="number"
            min="1"
            max="50"
            defaultValue={settings.max_sources_per_topic}
            onBlur={(e) => {
              const value = parseInt(e.target.value);
              if (!isNaN(value) && value >= 1 && value <= 50) {
                if (value !== settings.max_sources_per_topic) {
                  autoSave('max_sources_per_topic', value);
                }
              } else {
                // Reset to previous valid value on invalid input
                e.target.value = settings.max_sources_per_topic.toString();
              }
            }}
            className="input w-full"
          />
        </div>

        <div className="flex items-center gap-3">
          <label className="relative inline-flex items-center cursor-pointer">
            <input
              type="checkbox"
              checked={settings.enable_notifications}
              onChange={async (e) => {
                const enabled = e.target.checked;
                // Request notification permission when enabling
                if (enabled) {
                  try {
                    await invoke<boolean>('request_notification_permission');
                  } catch (err) {
                    console.error('Failed to request notification permission:', err);
                  }
                }
                autoSave('enable_notifications', enabled);
              }}
              className="sr-only peer"
            />
            <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-primary-300 dark:peer-focus:ring-primary-800 rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all dark:border-gray-600 peer-checked:bg-primary-600"></div>
          </label>
          <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
            Enable Notifications
          </span>
          {savedIndicator === 'enable_notifications' && (
            <motion.span
              initial={{ opacity: 0, scale: 0.8 }}
              animate={{ opacity: 1, scale: 1 }}
              exit={{ opacity: 0 }}
              className="text-xs text-green-600 dark:text-green-400 flex items-center gap-1"
            >
              <CheckCircle2 className="w-3 h-3" /> Saved
            </motion.span>
          )}
        </div>

        <div className="flex items-center gap-3">
          <label className="relative inline-flex items-center cursor-pointer">
            <input
              type="checkbox"
              checked={settings.notification_sound ?? true}
              onChange={(e) => autoSave('notification_sound', e.target.checked)}
              disabled={!settings.enable_notifications}
              className="sr-only peer"
            />
            <div className={`w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-primary-300 dark:peer-focus:ring-primary-800 rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all dark:border-gray-600 peer-checked:bg-primary-600 ${!settings.enable_notifications ? 'opacity-50' : ''}`}></div>
          </label>
          <span className={`text-sm font-medium text-gray-700 dark:text-gray-300 ${!settings.enable_notifications ? 'opacity-50' : ''}`}>
            Notification Sound
          </span>
          {savedIndicator === 'notification_sound' && (
            <motion.span
              initial={{ opacity: 0, scale: 0.8 }}
              animate={{ opacity: 1, scale: 1 }}
              exit={{ opacity: 0 }}
              className="text-xs text-green-600 dark:text-green-400 flex items-center gap-1"
            >
              <CheckCircle2 className="w-3 h-3" /> Saved
            </motion.span>
          )}
        </div>

        <div className="pt-6 border-t border-gray-200 dark:border-gray-700">
          <MagneticButton
            onClick={handleRunNow}
            disabled={running}
            variant="primary"
            className="flex items-center gap-2"
          >
            {running ? (
              <Loader2 className="w-4 h-4 animate-spin" />
            ) : (
              <Play className="w-4 h-4" />
            )}
            Run Research Now
          </MagneticButton>
        </div>
      </div>
    </div>
  );
}
