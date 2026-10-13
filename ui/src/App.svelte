<script lang="ts">
  import { onMount } from 'svelte';
  import type { ProfileConfig, Plan, RunReport, RulePredicate, AttachmentMode, HistoryEntry } from './types';
  import {
    listProfiles,
    saveProfile,
    deleteProfile,
    previewProfile,
    applyProfile,
    reconcileProfile,
    getRunHistory,
    getDiagnostics,
  } from './api';

  let profiles: ProfileConfig[] = [];
  let selectedProfile: ProfileConfig | null = null;
  let isEditing = false;
  let isPreviewing = false;
  let isViewingHistory = false;
  let historyList: HistoryEntry[] = [];
  let activePlan: Plan | null = null;
  let lastReport: RunReport | null = null;
  let diagnosticsData: any = null;
  let loading = false;
  let message = '';

  // Theme state: 'light' | 'dark'
  let currentTheme: 'light' | 'dark' = 'light';

  // Form state
  let editId = '';
  let editName = '';
  let editSource = '';
  let editDest = '';
  let editScopes = '';
  let editExcludes = '';
  let editRuleProp = 'status';
  let editRuleOp: any = 'equals';
  let editRuleVal = 'completed';
  let editAttachmentMode: AttachmentMode = 'referenced';

  onMount(async () => {
    // Detect and apply initial theme
    const savedTheme = localStorage.getItem('vault_publisher_theme');
    if (savedTheme === 'dark' || savedTheme === 'light') {
      currentTheme = savedTheme;
    } else if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {
      currentTheme = 'dark';
    } else {
      currentTheme = 'light';
    }
    applyTheme(currentTheme);

    await refreshProfiles();
  });

  function applyTheme(t: 'light' | 'dark') {
    currentTheme = t;
    document.documentElement.setAttribute('data-theme', t);
    localStorage.setItem('vault_publisher_theme', t);
  }

  function toggleTheme() {
    applyTheme(currentTheme === 'light' ? 'dark' : 'light');
  }

  async function refreshProfiles() {
    loading = true;
    try {
      profiles = await listProfiles();
      if (profiles.length === 0) {
        // Seed an initial demo profile if empty
        const initial: ProfileConfig = {
          id: 'shared-notes',
          name: 'Shared Notes',
          sourceVault: '/Users/notes/Private Vault',
          destinationVault: '/Users/notes/Shared Vault',
          scopes: ['Courses', 'Projects/Public'],
          excludes: ['.obsidian', '_private', '*.draft.md'],
          rules: {
            all: [
              { property: 'status', operator: 'equals', value: 'completed' },
              { property: 'share', operator: 'equals', value: true },
            ],
          },
          attachments: { mode: 'referenced' },
        };
        await saveProfile(initial);
        profiles = await listProfiles();
      }
    } catch (e: any) {
      message = 'Failed to load profiles: ' + e;
    } finally {
      loading = false;
    }
  }

  function startCreate() {
    editId = 'profile-' + Date.now();
    editName = 'New Profile';
    editSource = '';
    editDest = '';
    editScopes = 'Courses';
    editExcludes = '.obsidian, _private, *.draft.md';
    editRuleProp = 'status';
    editRuleOp = 'equals';
    editRuleVal = 'completed';
    editAttachmentMode = 'referenced';
    isEditing = true;
    isPreviewing = false;
  }

  function startEdit(p: ProfileConfig) {
    editId = p.id;
    editName = p.name;
    editSource = p.sourceVault;
    editDest = p.destinationVault;
    editScopes = p.scopes.join(', ');
    editExcludes = (p.excludes || []).join(', ');
    const firstRule = p.rules?.all?.[0];
    editRuleProp = firstRule?.property || 'status';
    editRuleOp = firstRule?.operator || 'equals';
    editRuleVal = firstRule?.value !== undefined ? String(firstRule.value) : 'completed';
    editAttachmentMode = p.attachments?.mode || 'referenced';
    isEditing = true;
    isPreviewing = false;
  }

  async function handleSave() {
    if (!editId || !editName || !editSource || !editDest) {
      alert('Please fill in Profile Name, Source Vault, and Destination Vault.');
      return;
    }

    const valParsed =
      editRuleVal === 'true' ? true : editRuleVal === 'false' ? false : editRuleVal;

    const profileToSave: ProfileConfig = {
      id: editId,
      name: editName,
      sourceVault: editSource,
      destinationVault: editDest,
      scopes: editScopes.split(',').map((s) => s.trim()).filter(Boolean),
      excludes: editExcludes.split(',').map((s) => s.trim()).filter(Boolean),
      rules: {
        all: [
          {
            property: editRuleProp,
            operator: editRuleOp,
            value: valParsed,
          },
        ],
      },
      attachments: { mode: editAttachmentMode },
    };

    try {
      await saveProfile(profileToSave);
      isEditing = false;
      message = `Saved profile '${profileToSave.name}' successfully.`;
      await refreshProfiles();
    } catch (e: any) {
      alert('Error saving profile: ' + e);
    }
  }

  async function handleDelete(id: string) {
    if (confirm(`Are you sure you want to delete profile '${id}'?`)) {
      await deleteProfile(id);
      await refreshProfiles();
    }
  }

  async function handlePreview(p: ProfileConfig) {
    loading = true;
    try {
      selectedProfile = p;
      activePlan = await previewProfile(p.id);
      isPreviewing = true;
      isEditing = false;
      lastReport = null;
    } catch (e: any) {
      alert('Error generating preview: ' + e);
    } finally {
      loading = false;
    }
  }

  async function handleApply(id: string) {
    loading = true;
    try {
      lastReport = await applyProfile(id);
      message = `Applied profile '${id}': ${lastReport.created_count} created, ${lastReport.updated_count} updated.`;
      // Refresh preview to show updated status
      if (selectedProfile) {
        activePlan = await previewProfile(selectedProfile.id);
      }
    } catch (e: any) {
      alert('Error applying plan: ' + e);
    } finally {
      loading = false;
    }
  }

  async function handleReconcile(id: string) {
    loading = true;
    try {
      lastReport = await reconcileProfile(id);
      message = `Reconciled '${id}': ${lastReport.created_count} created, ${lastReport.updated_count} updated, ${lastReport.archived_count} archived.`;
    } catch (e: any) {
      alert('Error reconciling: ' + e);
    } finally {
      loading = false;
    }
  }

  async function showDiagnostics() {
    diagnosticsData = await getDiagnostics();
  }

  async function showHistory() {
    historyList = await getRunHistory();
    isViewingHistory = true;
  }

  function exportConfig() {
    const blob = new Blob([JSON.stringify({ version: 1, profiles }, null, 2)], {
      type: 'application/json',
    });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'vault-publisher-config.json';
    a.click();
  }

  function getActionKey(action: any): string {
    return Object.keys(action)[0];
  }

  function getActionPath(action: any): string {
    const key = getActionKey(action);
    return action[key]?.relative_path || '';
  }

  function getActionReason(action: any): string {
    const key = getActionKey(action);
    return action[key]?.reason || '';
  }
</script>

<main class="app-container">
  <header class="header">
    <div class="logo-area">
      <div class="logo-badge">VP</div>
      <div>
        <h1>Vault Publisher</h1>
        <p class="subtitle">Controlled rule-based publishing for Obsidian vaults</p>
      </div>
    </div>
    <div class="header-actions">
      <button class="btn btn-secondary theme-toggle-btn" on:click={toggleTheme} title="Switch Light / Dark Theme">
        {#if currentTheme === 'dark'}
          🌙 Dark Mode
        {:else}
          ☀️ Light Mode
        {/if}
      </button>
      <button class="btn btn-secondary" on:click={showHistory}>Activity Log</button>
      <button class="btn btn-secondary" on:click={exportConfig}>Export Config</button>
      <button class="btn btn-secondary" on:click={showDiagnostics}>Diagnostics</button>
      <button class="btn btn-primary" on:click={startCreate}>+ New Profile</button>
    </div>
  </header>

  {#if message}
    <div class="alert alert-info">
      <span>{message}</span>
      <button class="close-btn" on:click={() => (message = '')}>×</button>
    </div>
  {/if}

  <div class="content-grid">
    <!-- PROFILES LIST -->
    <section class="card profile-list-card">
      <div class="card-header">
        <h2>Publishing Profiles</h2>
        <span class="count-tag">{profiles.length} Active</span>
      </div>

      <div class="profiles-list">
        {#each profiles as p}
          <div class="profile-item {selectedProfile?.id === p.id ? 'active' : ''}">
            <div class="profile-info">
              <div class="profile-title-row">
                <h3>{p.name}</h3>
                <span class="badge badge-active">Active</span>
              </div>
              <div class="path-details">
                <div><span class="path-label">From:</span> <code>{p.sourceVault}</code></div>
                <div><span class="path-label">To:</span> <code>{p.destinationVault}</code></div>
              </div>
              <div class="rule-summary">
                <strong>Rule:</strong> {p.rules?.all?.[0]?.property || 'status'} {p.rules?.all?.[0]?.operator || 'equals'} {JSON.stringify(p.rules?.all?.[0]?.value ?? 'completed')}
                &bull; <strong>Scope:</strong> {p.scopes.join(', ')}
              </div>
            </div>

            <div class="profile-actions">
              <button class="btn btn-outline" on:click={() => handlePreview(p)}>Preview</button>
              <button class="btn btn-outline" on:click={() => handleReconcile(p.id)}>Reconcile</button>
              <button class="btn btn-outline" on:click={() => startEdit(p)}>Edit</button>
              <button class="btn btn-danger-outline" on:click={() => handleDelete(p.id)}>Delete</button>
            </div>
          </div>
        {/each}

        {#if profiles.length === 0}
          <div class="empty-state">
            <p>No publishing profiles configured yet.</p>
            <button class="btn btn-primary" style="margin-top: 10px;" on:click={startCreate}>Create your first profile</button>
          </div>
        {/if}
      </div>
    </section>

    <!-- RIGHT PANEL: EDIT FORM OR PREVIEW -->
    <section class="card detail-card">
      {#if isEditing}
        <div class="card-header">
          <h2>{editId.startsWith('profile-') ? 'Create Profile' : 'Edit Profile'}</h2>
          <button class="btn btn-ghost" on:click={() => (isEditing = false)}>Cancel</button>
        </div>

        <form on:submit|preventDefault={handleSave}>
          <div class="form-group">
            <label for="name">Profile Name</label>
            <input id="name" bind:value={editName} placeholder="e.g. Shared Notes" />
          </div>

          <div class="form-group">
            <label for="source">Source Vault Path (Private)</label>
            <input id="source" bind:value={editSource} placeholder="/Users/me/Documents/MyNotes" />
          </div>

          <div class="form-group">
            <label for="dest">Destination Vault Path (Shared)</label>
            <input id="dest" bind:value={editDest} placeholder="/Users/me/Documents/SharedNotes" />
          </div>

          <div class="form-group">
            <label for="scopes">Scope Folders (comma-separated)</label>
            <input id="scopes" bind:value={editScopes} placeholder="Courses, Projects/Public" />
            <span class="hint">Leave empty to include all non-excluded folders</span>
          </div>

          <div class="form-group">
            <label for="excludes">Exclusions (comma-separated patterns)</label>
            <input id="excludes" bind:value={editExcludes} placeholder=".obsidian, _private, *.draft.md" />
          </div>

          <div class="form-box">
            <h3>Publishing Filter Rule</h3>
            <p class="rule-hint">Notes are only published if frontmatter matches this predicate:</p>
            <div class="form-row">
              <div class="flex-1">
                <label for="prop">Frontmatter Field</label>
                <input id="prop" bind:value={editRuleProp} placeholder="status" />
              </div>
              <div>
                <label for="op">Operator</label>
                <select id="op" bind:value={editRuleOp}>
                  <option value="equals">equals</option>
                  <option value="notEquals">not equals</option>
                  <option value="contains">contains</option>
                  <option value="exists">exists</option>
                  <option value="notExists">not exists</option>
                </select>
              </div>
              <div class="flex-1">
                <label for="val">Expected Value</label>
                <input id="val" bind:value={editRuleVal} placeholder="completed or true" />
              </div>
            </div>
          </div>

          <div class="form-group">
            <label for="att-mode">Attachment Strategy</label>
            <select id="att-mode" bind:value={editAttachmentMode}>
              <option value="referenced">Referenced only (Copy images/files linked in published notes)</option>
              <option value="explicit">Explicit only (Copy attachments explicitly inside scope)</option>
              <option value="markdown-only">Markdown only (Do not copy attachments)</option>
            </select>
          </div>

          <div class="form-actions">
            <button type="button" class="btn btn-secondary" on:click={() => (isEditing = false)}>Cancel</button>
            <button type="submit" class="btn btn-primary">Save Profile</button>
          </div>
        </form>
      {:else if isPreviewing && activePlan}
        <div class="card-header">
          <div>
            <h2>Plan Preview: {selectedProfile?.name}</h2>
            <span class="subtitle">{activePlan.actions.length} planned operations</span>
          </div>
          <div style="display: flex; gap: 8px;">
            <button class="btn btn-primary" on:click={() => activePlan && handleApply(activePlan.profile_id)}>Apply Plan Now</button>
            <button class="btn btn-ghost" on:click={() => (isPreviewing = false)}>Close</button>
          </div>
        </div>

        {#if lastReport}
          <div class="report-banner">
            <strong>Execution Completed:</strong>
            +{lastReport.created_count} created, ~{lastReport.updated_count} updated,
            -{lastReport.archived_count} archived, {lastReport.skipped_count} skipped
            {#if lastReport.conflict_count > 0}
              <span class="text-danger">&bull; {lastReport.conflict_count} conflicts preserved</span>
            {/if}
          </div>
        {/if}

        <div class="plan-actions-list">
          {#each activePlan.actions as action}
            {@const key = getActionKey(action)}
            <div class="plan-row">
              <span class="action-tag tag-{key.toLowerCase()}">{key}</span>
              <span class="action-path">{getActionPath(action)}</span>
              {#if getActionReason(action)}
                <span class="action-reason">{getActionReason(action)}</span>
              {/if}
            </div>
          {/each}
          {#if activePlan.actions.length === 0}
            <div class="empty-state">
              <p>Everything is currently up to date. No actions pending.</p>
            </div>
          {/if}
        </div>
      {:else}
        <div class="placeholder-state">
          <div class="placeholder-icon">📂</div>
          <h3>Select a Profile</h3>
          <p>Click "Preview" on any profile to simulate publishing diffs without writing files, or click "+ New Profile" to create a new sync mapping.</p>
        </div>
      {/if}
    </section>
  </div>

  <!-- DIAGNOSTICS MODAL -->
  {#if diagnosticsData}
    <div class="modal-backdrop" on:click={() => (diagnosticsData = null)} on:keydown={(e) => e.key === 'Escape' && (diagnosticsData = null)} role="presentation">
      <div class="modal" role="dialog" aria-modal="true" tabindex="-1" on:click|stopPropagation on:keydown|stopPropagation>
        <h2>System Diagnostics</h2>
        <pre>{JSON.stringify(diagnosticsData, null, 2)}</pre>
        <button class="btn btn-secondary" on:click={() => (diagnosticsData = null)}>Close</button>
      </div>
    </div>
  {/if}

  <!-- HISTORY MODAL -->
  {#if isViewingHistory}
    <div class="modal-backdrop" on:click={() => (isViewingHistory = false)} on:keydown={(e) => e.key === 'Escape' && (isViewingHistory = false)} role="presentation">
      <div class="modal" role="dialog" aria-modal="true" tabindex="-1" style="max-width: 650px;" on:click|stopPropagation on:keydown|stopPropagation>
        <h2>Recent Execution History</h2>
        {#if historyList.length === 0}
          <p style="color: var(--text-muted); margin: 20px 0;">No run history entries recorded yet.</p>
        {:else}
          <div style="max-height: 380px; overflow-y: auto; margin-bottom: 16px;">
            <table style="width: 100%; border-collapse: collapse; font-size: 0.85rem; color: var(--text-primary);">
              <thead>
                <tr style="border-bottom: 2px solid var(--border-color); text-align: left; color: var(--text-muted);">
                  <th style="padding: 6px;">Profile</th>
                  <th style="padding: 6px;">Created</th>
                  <th style="padding: 6px;">Updated</th>
                  <th style="padding: 6px;">Archived</th>
                  <th style="padding: 6px;">Conflicts</th>
                  <th style="padding: 6px;">Duration</th>
                </tr>
              </thead>
              <tbody>
                {#each historyList as h}
                  <tr style="border-bottom: 1px solid var(--border-color);">
                    <td style="padding: 6px;"><strong>{h.profileId}</strong></td>
                    <td style="padding: 6px; color: var(--badge-create-text);">+{h.created}</td>
                    <td style="padding: 6px; color: var(--badge-update-text);">~{h.updated}</td>
                    <td style="padding: 6px; color: var(--badge-archive-text);">-{h.archived}</td>
                    <td style="padding: 6px; color: var(--badge-conflict-text);">{h.conflicts}</td>
                    <td style="padding: 6px; color: var(--text-muted);">{h.durationMs}ms</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
        <button class="btn btn-secondary" on:click={() => (isViewingHistory = false)}>Close</button>
      </div>
    </div>
  {/if}
</main>

<style>
  .app-container {
    max-width: 1280px;
    margin: 0 auto;
    padding: 24px;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 24px;
    padding-bottom: 16px;
    border-bottom: 1px solid var(--border-color);
  }

  .logo-area {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .logo-badge {
    width: 44px;
    height: 44px;
    background: var(--accent);
    color: white;
    font-weight: 700;
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1.2rem;
  }

  h1 {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--text-h);
  }

  .subtitle {
    margin: 4px 0 0;
    color: var(--text-muted);
    font-size: 0.9rem;
  }

  .header-actions {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .theme-toggle-btn {
    font-weight: 600;
  }

  .content-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 20px;
  }

  @media (max-width: 900px) {
    .content-grid {
      grid-template-columns: 1fr;
    }
  }

  .card {
    background: var(--bg-card);
    border-radius: 12px;
    border: 1px solid var(--border-color);
    padding: 20px;
    box-shadow: var(--shadow);
    color: var(--text-primary);
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
  }

  .card-header h2 {
    margin: 0;
    font-size: 1.2rem;
    font-weight: 600;
    color: var(--text-h);
  }

  .count-tag {
    background: var(--accent-light);
    color: var(--accent-text);
    padding: 3px 10px;
    border-radius: 12px;
    font-size: 0.8rem;
    font-weight: 600;
  }

  .profiles-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .profile-item {
    border: 1px solid var(--border-color);
    background: var(--bg-card);
    border-radius: 8px;
    padding: 14px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    transition: all 0.15s ease-in-out;
  }

  .profile-item.active {
    border-color: var(--border-active);
    background-color: var(--bg-card-active);
  }

  .profile-title-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .profile-title-row h3 {
    margin: 0;
    font-size: 1.05rem;
    font-weight: 600;
    color: var(--text-h);
  }

  .badge-active {
    background: var(--badge-create-bg);
    color: var(--badge-create-text);
    padding: 2px 8px;
    border-radius: 12px;
    font-size: 0.75rem;
    font-weight: 600;
  }

  .path-details {
    font-size: 0.85rem;
    margin: 6px 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .path-label {
    color: var(--text-muted);
    font-weight: 600;
    margin-right: 6px;
  }

  code {
    background: var(--code-bg);
    color: var(--code-text);
    padding: 2px 6px;
    border-radius: 4px;
    font-family: var(--font-mono);
    font-size: 0.8rem;
    border: 1px solid var(--border-color);
  }

  .rule-summary {
    font-size: 0.85rem;
    color: var(--text-secondary);
  }

  .profile-actions {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }

  .btn {
    padding: 6px 14px;
    border-radius: 6px;
    font-size: 0.85rem;
    font-weight: 500;
    cursor: pointer;
    border: 1px solid transparent;
    transition: all 0.15s ease;
  }

  .btn-primary {
    background: var(--accent);
    color: white;
  }
  .btn-primary:hover {
    background: var(--accent-hover);
  }

  .btn-secondary {
    background: var(--btn-sec-bg);
    color: var(--btn-sec-text);
    border-color: var(--btn-sec-border);
  }
  .btn-secondary:hover {
    background: var(--btn-sec-hover);
  }

  .btn-outline {
    background: var(--btn-outline-bg);
    border-color: var(--btn-outline-border);
    color: var(--btn-outline-text);
  }
  .btn-outline:hover {
    background: var(--btn-outline-hover);
  }

  .btn-danger-outline {
    background: var(--bg-card);
    border-color: #fca5a5;
    color: #ef4444;
  }
  .btn-danger-outline:hover {
    background: var(--badge-conflict-bg);
  }

  .btn-ghost {
    background: transparent;
    color: var(--text-muted);
  }
  .btn-ghost:hover {
    color: var(--text-primary);
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-bottom: 14px;
  }

  .form-group label {
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--text-secondary);
  }

  input, select {
    padding: 8px 10px;
    background: var(--input-bg);
    color: var(--input-text);
    border: 1px solid var(--input-border);
    border-radius: 6px;
    font-size: 0.9rem;
    font-family: inherit;
  }

  input:focus, select:focus {
    outline: none;
    border-color: var(--accent);
  }

  .form-row {
    display: flex;
    gap: 10px;
    align-items: center;
  }

  .flex-1 { flex: 1; }

  .hint {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .form-box {
    background: var(--bg-card-subtle);
    border: 1px solid var(--border-color);
    padding: 12px;
    border-radius: 8px;
    margin-bottom: 14px;
  }

  .form-box h3 {
    margin: 0 0 4px;
    font-size: 0.95rem;
    color: var(--text-h);
  }

  .rule-hint {
    margin: 0 0 10px;
    font-size: 0.8rem;
    color: var(--text-muted);
  }

  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 16px;
  }

  .plan-actions-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
    max-height: 480px;
    overflow-y: auto;
  }

  .plan-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 12px;
    border-radius: 6px;
    font-size: 0.85rem;
    background: var(--bg-card-subtle);
    border: 1px solid var(--border-color);
    color: var(--text-primary);
  }

  .tag-create { background: var(--badge-create-bg); color: var(--badge-create-text); }
  .tag-update { background: var(--badge-update-bg); color: var(--badge-update-text); }
  .tag-skip { background: var(--badge-skip-bg); color: var(--badge-skip-text); }
  .tag-archive { background: var(--badge-archive-bg); color: var(--badge-archive-text); }
  .tag-conflict { background: var(--badge-conflict-bg); color: var(--badge-conflict-text); font-weight: 700; }
  .tag-warn { background: var(--badge-warn-bg); color: var(--badge-warn-text); }

  .action-tag {
    font-size: 0.75rem;
    padding: 2px 6px;
    border-radius: 4px;
    font-weight: 600;
    min-width: 60px;
    text-align: center;
  }

  .action-path {
    font-family: var(--font-mono);
    flex: 1;
    color: var(--text-primary);
  }

  .action-reason {
    font-size: 0.75rem;
    color: var(--badge-conflict-text);
  }

  .report-banner {
    background: var(--badge-create-bg);
    border: 1px solid var(--border-color);
    color: var(--badge-create-text);
    padding: 10px 14px;
    border-radius: 6px;
    margin-bottom: 14px;
    font-size: 0.85rem;
  }

  .text-danger { color: var(--badge-conflict-text); font-weight: 600; }

  .placeholder-state {
    text-align: center;
    padding: 60px 20px;
    color: var(--text-muted);
  }

  .placeholder-state h3 {
    color: var(--text-h);
  }

  .placeholder-icon {
    font-size: 2.5rem;
    margin-bottom: 12px;
  }

  .empty-state {
    text-align: center;
    padding: 30px;
    color: var(--text-muted);
  }

  .alert {
    padding: 10px 14px;
    border-radius: 6px;
    margin-bottom: 16px;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .alert-info {
    background: var(--badge-update-bg);
    border: 1px solid var(--border-color);
    color: var(--badge-update-text);
  }

  .close-btn {
    background: none;
    border: none;
    font-size: 1.2rem;
    cursor: pointer;
    color: inherit;
  }

  .modal-backdrop {
    position: fixed;
    top: 0; left: 0; right: 0; bottom: 0;
    background: var(--modal-overlay);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 999;
  }

  .modal {
    background: var(--modal-bg);
    color: var(--text-primary);
    padding: 24px;
    border-radius: 12px;
    max-width: 500px;
    width: 100%;
    border: 1px solid var(--border-color);
    box-shadow: var(--shadow);
  }

  .modal h2 {
    color: var(--text-h);
    margin-top: 0;
  }

  pre {
    background: var(--code-bg);
    color: var(--code-text);
    border: 1px solid var(--border-color);
    padding: 12px;
    border-radius: 6px;
    overflow-x: auto;
    font-size: 0.8rem;
    font-family: var(--font-mono);
  }
</style>
