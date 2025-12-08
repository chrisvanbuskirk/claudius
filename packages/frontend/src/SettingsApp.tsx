import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { X, ArrowLeft } from 'lucide-react';
import { SettingsPage } from './pages/SettingsPage';

export function SettingsApp() {
  const handleClose = async () => {
    // Hide this settings window
    const currentWindow = getCurrentWindow();
    await currentWindow.hide();
  };

  const handleBackToMain = async () => {
    // Open main window and close settings
    try {
      await invoke('open_main_window');
      const currentWindow = getCurrentWindow();
      await currentWindow.hide();
    } catch (e) {
      console.error('Failed to open main window:', e);
    }
  };

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      <div className="p-4">
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center gap-3">
            <button
              onClick={handleBackToMain}
              className="p-2 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 hover:bg-gray-200 dark:hover:bg-gray-700 rounded-lg transition-colors"
              title="Back to main window"
            >
              <ArrowLeft className="w-5 h-5" />
            </button>
            <h1 className="text-xl font-semibold text-gray-900 dark:text-white">
              Claudius Settings
            </h1>
          </div>
          <button
            onClick={handleClose}
            className="p-2 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 hover:bg-gray-200 dark:hover:bg-gray-700 rounded-lg transition-colors"
            title="Close settings"
          >
            <X className="w-5 h-5" />
          </button>
        </div>
        <SettingsPage />
      </div>
    </div>
  );
}
