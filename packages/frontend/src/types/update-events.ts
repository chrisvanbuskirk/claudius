// Event: Update available
export interface UpdateAvailableEvent {
  version: string;
  notes: string | null;
  date: string | null;
}

// Event: Update download progress
export interface UpdateProgressEvent {
  downloaded: number;
  total: number | null;
}

// Event: Update downloaded and ready to install
export interface UpdateDownloadedEvent {
  version: string;
}

// Local state for update tracking
export interface UpdateState {
  isChecking: boolean;
  updateAvailable: boolean;
  updateDownloaded: boolean;
  version: string | null;
  notes: string | null;
  downloadProgress: number; // 0-100
  error: string | null;
}

// Response from check_for_update command
export interface UpdateInfo {
  version: string;
  notes: string | null;
  date: string | null;
}
