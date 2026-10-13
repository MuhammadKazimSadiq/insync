import type { ProfileConfig, Plan, RunReport } from './types';

function isTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

export async function listProfiles(): Promise<ProfileConfig[]> {
  if (isTauri()) {
    const { invoke } = await import('@tauri-apps/api/core');
    return await invoke('list_profiles');
  }
  const data = localStorage.getItem('vault_publisher_profiles');
  return data ? JSON.parse(data) : [];
}

export async function saveProfile(profile: ProfileConfig): Promise<void> {
  if (isTauri()) {
    const { invoke } = await import('@tauri-apps/api/core');
    return await invoke('save_profile', { profile });
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
    const { invoke } = await import('@tauri-apps/api/core');
    return await invoke('delete_profile', { profileId });
  }
  let profiles = await listProfiles();
  profiles = profiles.filter((p) => p.id !== profileId);
  localStorage.setItem('vault_publisher_profiles', JSON.stringify(profiles));
}

export async function previewProfile(profileId: string): Promise<Plan> {
  if (isTauri()) {
    const { invoke } = await import('@tauri-apps/api/core');
    return await invoke('preview_profile', { profileId });
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
    const { invoke } = await import('@tauri-apps/api/core');
    return await invoke('apply_profile', { profileId });
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
    const { invoke } = await import('@tauri-apps/api/core');
    return await invoke('reconcile_profile', { profileId });
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
    const { invoke } = await import('@tauri-apps/api/core');
    return await invoke('get_run_history');
  }
  const data = localStorage.getItem('vault_publisher_history');
  return data ? JSON.parse(data) : [];
}

export async function getDiagnostics(): Promise<any> {
  if (isTauri()) {
    const { invoke } = await import('@tauri-apps/api/core');
    return await invoke('get_diagnostics');
  }
  return {
    version: '0.1.0-web',
    mode: 'Browser Sandbox',
    os: navigator.userAgent,
    profilesCount: (await listProfiles()).length,
  };
}
