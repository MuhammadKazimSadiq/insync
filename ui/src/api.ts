import type { ProfileConfig, Plan, RunReport, HistoryEntry } from './types';

function isTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

export async function listProfiles(): Promise<ProfileConfig[]> {
  if (isTauri()) {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const res = await invoke<ProfileConfig[]>('list_profiles');
      return res;
    } catch (err) {
      console.warn('[VaultPublisher] Tauri list_profiles failed, falling back to local storage:', err);
    }
  }
  const data = localStorage.getItem('vault_publisher_profiles');
  return data ? JSON.parse(data) : [];
}

export async function saveProfile(profile: ProfileConfig): Promise<void> {
  if (isTauri()) {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('save_profile', { profile });
      return;
    } catch (err) {
      console.warn('[VaultPublisher] Tauri save_profile failed, saving locally:', err);
    }
  }
  const profiles = await listProfiles();
  const idx = profiles.findIndex((p) => p.id === profile.id);
  if (idx >= 0) {
    profiles[idx] = profile;
  } else {
    profiles.push(profile);
  }
  localStorage.setItem('vault_publisher_profiles', JSON.stringify(profiles));
}

export async function deleteProfile(profileId: string): Promise<void> {
  if (isTauri()) {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('delete_profile', { profileId });
      return;
    } catch (err) {
      console.warn('[VaultPublisher] Tauri delete_profile failed, deleting locally:', err);
    }
  }
  let profiles = await listProfiles();
  profiles = profiles.filter((p) => p.id !== profileId);
  localStorage.setItem('vault_publisher_profiles', JSON.stringify(profiles));
}

export async function previewProfile(profileId: string): Promise<Plan> {
  if (isTauri()) {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      return await invoke<Plan>('preview_profile', { profileId });
    } catch (err) {
      console.warn('[VaultPublisher] Tauri preview_profile failed, using preview fallback:', err);
    }
  }
  return {
    profile_id: profileId,
    source_vault: '/Users/mock/Notes/Private',
    destination_vault: '/Users/mock/Notes/Shared',
    actions: [
      { Create: { relative_path: 'Courses/Lesson 1.md' } },
      { Skip: { relative_path: 'Courses/Overview.md' } },
      { Conflict: { relative_path: 'Courses/Syllabus.md', reason: 'Destination file modified externally' } },
    ],
  };
}

export async function applyProfile(profileId: string): Promise<RunReport> {
  if (isTauri()) {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      return await invoke<RunReport>('apply_profile', { profileId });
    } catch (err) {
      console.warn('[VaultPublisher] Tauri apply_profile failed, using mock report:', err);
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
      const { invoke } = await import('@tauri-apps/api/core');
      return await invoke<RunReport>('reconcile_profile', { profileId });
    } catch (err) {
      console.warn('[VaultPublisher] Tauri reconcile_profile failed, using mock report:', err);
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
      const { invoke } = await import('@tauri-apps/api/core');
      return await invoke<HistoryEntry[]>('get_run_history');
    } catch (err) {
      console.warn('[VaultPublisher] Tauri get_run_history failed, reading local storage:', err);
    }
  }
  const data = localStorage.getItem('vault_publisher_history');
  return data ? JSON.parse(data) : [];
}

export async function getDiagnostics(): Promise<any> {
  if (isTauri()) {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      return await invoke('get_diagnostics');
    } catch (err) {
      console.warn('[VaultPublisher] Tauri get_diagnostics failed, using browser diagnostics:', err);
    }
  }
  return {
    version: '0.1.0-web',
    mode: 'Browser Sandbox',
    os: typeof navigator !== 'undefined' ? navigator.userAgent : 'Unknown',
    profilesCount: (await listProfiles()).length,
  };
}
