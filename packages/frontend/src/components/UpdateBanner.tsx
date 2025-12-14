import { Download, RefreshCw, X } from 'lucide-react';
import { useUpdater } from '../hooks/useUpdater';

export function UpdateBanner() {
  const {
    updateAvailable,
    updateDownloaded,
    version,
    notes,
    downloadProgress,
    installAndRestart,
    dismiss,
  } = useUpdater();

  // Don't render if no update
  if (!updateAvailable && !updateDownloaded) {
    return null;
  }

  const isDownloading = updateAvailable && !updateDownloaded && downloadProgress > 0 && downloadProgress < 100;

  return (
    <div className="mb-6">
      <div className="card p-4 bg-blue-50 dark:bg-blue-900/20 border-blue-200 dark:border-blue-800">
        <div className="flex items-start gap-3">
          {updateDownloaded ? (
            <RefreshCw className="w-5 h-5 text-blue-600 dark:text-blue-400 flex-shrink-0 mt-0.5" />
          ) : (
            <Download className="w-5 h-5 text-blue-600 dark:text-blue-400 flex-shrink-0 mt-0.5" />
          )}
          <div className="flex-1 min-w-0">
            <h3 className="font-semibold text-blue-900 dark:text-blue-300 mb-1">
              {updateDownloaded
                ? `Update Ready - v${version}`
                : `Update Available - v${version}`}
            </h3>
            <p className="text-sm text-blue-800 dark:text-blue-400 mb-2">
              {updateDownloaded
                ? 'A new version has been downloaded and is ready to install.'
                : isDownloading
                  ? `Downloading update... ${downloadProgress}%`
                  : 'A new version is being downloaded in the background.'}
            </p>
            {notes && (
              <p className="text-xs text-blue-600/70 dark:text-blue-500/70 mb-2 line-clamp-2">
                {notes}
              </p>
            )}
            {updateDownloaded && (
              <button
                onClick={installAndRestart}
                className="text-sm font-medium text-blue-700 dark:text-blue-300 hover:underline flex items-center gap-1"
              >
                <RefreshCw className="w-3 h-3" />
                Restart to Update
              </button>
            )}
            {isDownloading && (
              <div className="w-full bg-blue-200 dark:bg-blue-800 rounded-full h-1.5 mt-2">
                <div
                  className="bg-blue-600 dark:bg-blue-400 h-1.5 rounded-full transition-all duration-300"
                  style={{ width: `${downloadProgress}%` }}
                />
              </div>
            )}
          </div>
          <button
            onClick={dismiss}
            className="p-1 rounded hover:bg-blue-200/50 dark:hover:bg-blue-800/50 text-blue-600 dark:text-blue-400"
            title="Dismiss"
          >
            <X className="w-4 h-4" />
          </button>
        </div>
      </div>
    </div>
  );
}
