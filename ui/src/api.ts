import { isTauri as checkTauri, invoke } from '@tauri-apps/api/core';
import type { ProfileConfig, Plan, RunReport, HistoryEntry } from './types';

export function isTauri(): boolean {
  if (typeof window === 'undefined') return false;
  return checkTauri() || '__TAURI_INTERNALS__' in window;
}

export async function listProfiles(): Promise<ProfileConfig[]> {
  if (isTauri()) {
    try {
      const res = await invoke<ProfileConfig[]>('list_profiles');
      return res;
    } catch (err) {
      console.error('[InSync] Tauri list_profiles failed:', err);
      throw err;
    }
  }
  const data = localStorage.getItem('insync_synced_folders') || localStorage.getItem('vault_publisher_profiles');
  return data ? JSON.parse(data) : [];
}

export async function saveProfile(profile: ProfileConfig): Promise<void> {
  if (isTauri()) {
    try {
      await invoke('save_profile', { profile });
      return;
    } catch (err) {
      console.error('[InSync] Tauri save_profile failed:', err);
      throw err;
    }
  }
  const profiles = await listProfiles();
  const idx = profiles.findIndex((p) => p.id === profile.id);
  if (idx >= 0) {
    profiles[idx] = profile;
  } else {
    profiles.push(profile);
  }
  localStorage.setItem('insync_synced_folders', JSON.stringify(profiles));
}

export async function deleteProfile(profileId: string): Promise<void> {
  if (isTauri()) {
    try {
      await invoke('delete_profile', { profileId });
      return;
    } catch (err) {
      console.error('[InSync] Tauri delete_profile failed:', err);
      throw err;
    }
  }
  let profiles = await listProfiles();
  profiles = profiles.filter((p) => p.id !== profileId);
  localStorage.setItem('insync_synced_folders', JSON.stringify(profiles));
}

export async function previewProfile(profileId: string): Promise<Plan> {
  if (isTauri()) {
    try {
      return await invoke<Plan>('preview_profile', { profileId });
    } catch (err) {
      console.error('[InSync] Tauri preview_profile failed:', err);
      throw err;
    }
  }
  return {
    profile_id: profileId,
    source_vault: '/Users/mock/Documents/SourceFolder',
    destination_vault: '/Users/mock/Documents/DestinationFolder',
    actions: [
      { Create: { relative_path: 'Notes/Document 1.md' } },
      { Skip: { relative_path: 'Notes/Overview.md' } },
      { Conflict: { relative_path: 'Notes/SharedSummary.md', reason: 'Destination file modified externally' } },
    ],
  };
}

export async function applyProfile(profileId: string): Promise<RunReport> {
  if (isTauri()) {
    try {
      return await invoke<RunReport>('apply_profile', { profileId });
    } catch (err) {
      console.error('[InSync] Tauri apply_profile failed:', err);
      throw err;
    }
  }
  return {
    profile_id: profileId,
    created_count: 1,
    updated_count: 0,
    archived_count: 0,
    skipped_count: 1,
    conflict_count: 1,
    warn_count: 0,
  };
}

export async function reconcileProfile(profileId: string): Promise<RunReport> {
  if (isTauri()) {
    try {
      return await invoke<RunReport>('reconcile_profile', { profileId });
    } catch (err) {
      console.error('[InSync] Tauri reconcile_profile failed:', err);
      throw err;
    }
  }
  return {
    profile_id: profileId,
    created_count: 1,
    updated_count: 0,
    archived_count: 0,
    skipped_count: 1,
    conflict_count: 1,
    warn_count: 0,
  };
}

export async function getRunHistory(): Promise<HistoryEntry[]> {
  if (isTauri()) {
    try {
      return await invoke<HistoryEntry[]>('get_run_history');
    } catch (err) {
      console.error('[InSync] Tauri get_run_history failed:', err);
      return [];
    }
  }
  const data = localStorage.getItem('insync_history');
  return data ? JSON.parse(data) : [];
}

export async function getDiagnostics(): Promise<any> {
  if (isTauri()) {
    try {
      return await invoke('get_diagnostics');
    } catch (err) {
      console.error('[InSync] Tauri get_diagnostics failed, using fallback:', err);
    }
  }
  return {
    app: 'InSync',
    version: '0.1.0-web',
    mode: 'Browser Sandbox',
    os: typeof navigator !== 'undefined' ? navigator.userAgent : 'Unknown',
    syncedFoldersCount: (await listProfiles()).length,
  };
}

export async function selectFolder(): Promise<string | null> {
  if (isTauri()) {
    try {
      const selected = await invoke<string | null>('select_folder');
      return selected;
    } catch (err) {
      console.error('[InSync] Tauri select_folder failed:', err);
    }
  }
  return prompt('Enter directory path:') || null;
}

export async function exportConfigFile(): Promise<string | null> {
  if (isTauri()) {
    try {
      const savedPath = await invoke<string | null>('export_config_file');
      return savedPath;
    } catch (err) {
      console.error('[InSync] Tauri export_config_file failed:', err);
    }
  }
  const profiles = await listProfiles();
  const blob = new Blob([JSON.stringify({ version: 1, profiles }, null, 2)], {
    type: 'application/json',
  });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = 'insync-config.json';
  a.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
  return 'insync-config.json';
}
