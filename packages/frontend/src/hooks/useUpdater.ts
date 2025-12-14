import { useEffect, useState, useCallback } from 'react';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import type {
  UpdateAvailableEvent,
  UpdateDownloadedEvent,
  UpdateProgressEvent,
  UpdateState,
  UpdateInfo,
} from '../types/update-events';

const initialState: UpdateState = {
  isChecking: false,
  updateAvailable: false,
  updateDownloaded: false,
  version: null,
  notes: null,
  downloadProgress: 0,
  error: null,
};

export function useUpdater() {
  const [state, setState] = useState<UpdateState>(initialState);

  useEffect(() => {
    let mounted = true;
    const unlistenPromises: Promise<UnlistenFn>[] = [];

    // Listen for update available event
    unlistenPromises.push(
      listen<UpdateAvailableEvent>('update:available', (event) => {
        if (!mounted) return;
        console.log('Update available:', event.payload);
        setState((prev) => ({
          ...prev,
          updateAvailable: true,
          version: event.payload.version,
          notes: event.payload.notes,
        }));
      })
    );

    // Listen for download progress
    unlistenPromises.push(
      listen<UpdateProgressEvent>('update:progress', (event) => {
        if (!mounted) return;
        const { downloaded, total } = event.payload;
        const progress = total ? Math.round((downloaded / total) * 100) : 0;
        setState((prev) => ({
          ...prev,
          downloadProgress: Math.min(progress, 100),
        }));
      })
    );

    // Listen for download complete
    unlistenPromises.push(
      listen<UpdateDownloadedEvent>('update:downloaded', (event) => {
        if (!mounted) return;
        console.log('Update downloaded:', event.payload);
        setState((prev) => ({
          ...prev,
          updateDownloaded: true,
          downloadProgress: 100,
        }));
      })
    );

    return () => {
      mounted = false;
      Promise.all(unlistenPromises).then((unlisteners) => {
        unlisteners.forEach((unlisten) => unlisten());
      });
    };
  }, []);

  // Manual check for updates
  const checkForUpdate = useCallback(async () => {
    setState((prev) => ({ ...prev, isChecking: true, error: null }));
    try {
      const result = await invoke<UpdateInfo | null>('check_for_update');
      if (result) {
        setState((prev) => ({
          ...prev,
          isChecking: false,
          updateAvailable: true,
          version: result.version,
          notes: result.notes,
        }));
        return result;
      } else {
        setState((prev) => ({ ...prev, isChecking: false }));
        return null;
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setState((prev) => ({
        ...prev,
        isChecking: false,
        error: errorMessage,
      }));
      throw err;
    }
  }, []);

  // Install update and restart
  const installAndRestart = useCallback(async () => {
    try {
      await invoke('install_update_and_restart');
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setState((prev) => ({
        ...prev,
        error: errorMessage,
      }));
      throw err;
    }
  }, []);

  // Dismiss the update banner
  const dismiss = useCallback(() => {
    setState((prev) => ({
      ...prev,
      updateAvailable: false,
      updateDownloaded: false,
    }));
  }, []);

  return {
    ...state,
    checkForUpdate,
    installAndRestart,
    dismiss,
  };
}
