<script lang="ts">
  import { onMount } from 'svelte';
  import logoSvg from './assets/logo.svg';
  import {
    Folder,
    FolderSync,
    Plus,
    Trash2,
    Play,
    CheckCircle2,
    AlertTriangle,
    Sun,
    Moon,
    FileText,
    Download,
    History,
    Activity,
    Settings,
    X,
    Check,
    Info
  } from '@lucide/svelte';
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
    selectFolder,
    exportConfigFile,
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
  let editAttachmentMode: AttachmentMode = 'referenced';
  let editRules: Array<{ property: string; operator: any; value: string }> = [];

  onMount(async () => {
    // Detect and apply initial theme
    const savedTheme = localStorage.getItem('insync_theme') || localStorage.getItem('vault_publisher_theme');
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
    localStorage.setItem('insync_theme', t);
  }

  function toggleTheme() {
    applyTheme(currentTheme === 'light' ? 'dark' : 'light');
  }

  function getRuleCount(rules: any): number {
    if (!rules) return 0;
    if (Array.isArray(rules)) return rules.length;
    if (Array.isArray(rules.all)) return rules.all.length;
    if (Array.isArray(rules.any)) return rules.any.length;
    if (rules.property) return 1;
    return 0;
  }

  async function refreshProfiles() {
    loading = true;
    try {
      profiles = await listProfiles();
    } catch (e: any) {
      console.error('[InSync] Failed to load profiles:', e);
      message = 'Failed to load synced folders: ' + (e?.message || e);
    } finally {
      loading = false;
    }
  }

  function startCreate() {
    editId = 'sync-' + Date.now();
    editName = '';
    editSource = '';
    editDest = '';
    editScopes = '';
    editExcludes = '.git, _private, *.draft.md';
    editAttachmentMode = 'explicit';
    editRules = [];
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
    editAttachmentMode = p.attachments?.mode || 'explicit';

    let rulesList: RulePredicate[] = [];
    if (p.rules) {
      if (Array.isArray(p.rules)) {
        rulesList = p.rules;
      } else if (Array.isArray((p.rules as any).all)) {
        rulesList = (p.rules as any).all;
      } else if (Array.isArray((p.rules as any).any)) {
        rulesList = (p.rules as any).any;
      } else if ((p.rules as any).property) {
        rulesList = [p.rules as any];
      }
    }

    if (rulesList.length > 0) {
      editRules = rulesList.map((r) => ({
        property: r.property || 'status',
        operator: r.operator || 'equals',
        value: r.value !== undefined ? String(r.value) : '',
      }));
    } else {
      editRules = [];
    }

    isEditing = true;
    isPreviewing = false;
  }

  function addRule() {
    editRules = [
      ...editRules,
      { property: 'status', operator: 'equals', value: 'completed' }
    ];
  }

  function removeRule(index: number) {
    editRules = editRules.filter((_, i) => i !== index);
  }

  async function pickSourceFolder() {
    const picked = await selectFolder();
    if (picked) {
      editSource = picked;
    }
  }

  async function pickDestFolder() {
    const picked = await selectFolder();
    if (picked) {
      editDest = picked;
    }
  }

  async function handleSave() {
    if (!editId || !editName || !editSource || !editDest) {
      alert('Please fill in Synced Folder Name, Source Folder, and Destination Folder.');
      return;
    }

    const mappedRules: RulePredicate[] = editRules
      .filter((r) => r.property.trim() !== '')
      .map((r) => {
        let parsedVal: any = r.value;
        if (r.value === 'true') parsedVal = true;
        else if (r.value === 'false') parsedVal = false;
        else if (!isNaN(Number(r.value)) && r.value.trim() !== '') parsedVal = Number(r.value);

        return {
          property: r.property.trim(),
          operator: r.operator,
          value: r.operator === 'exists' || r.operator === 'notExists' ? undefined : parsedVal,
        };
      });

    const profileToSave: ProfileConfig = {
      id: editId,
      name: editName,
      sourceVault: editSource,
      destinationVault: editDest,
      scopes: editScopes.split(',').map((s) => s.trim()).filter(Boolean),
      excludes: editExcludes.split(',').map((s) => s.trim()).filter(Boolean),
      rules: mappedRules.length > 0 ? { all: mappedRules } : null,
      attachments: { mode: editAttachmentMode },
    };

    try {
      loading = true;
      await saveProfile(profileToSave);
      isEditing = false;
      message = `Saved synced folder '${profileToSave.name}' successfully.`;
      await refreshProfiles();
      selectedProfile = profiles.find((p) => p.id === profileToSave.id) || profileToSave;
    } catch (e: any) {
      console.error('[InSync] Save error:', e);
      alert('Error saving synced folder: ' + (e?.message || e));
    } finally {
      loading = false;
    }
  }

  async function handleDelete(id: string) {
    if (confirm(`Are you sure you want to delete synced folder '${id}'?`)) {
      try {
        loading = true;
        await deleteProfile(id);
        if (selectedProfile?.id === id) {
          selectedProfile = null;
          isPreviewing = false;
        }
        message = `Deleted synced folder '${id}'.`;
        await refreshProfiles();
      } catch (e: any) {
        console.error('[InSync] Delete error:', e);
        alert('Error deleting synced folder: ' + (e?.message || e));
      } finally {
        loading = false;
      }
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
      message = `Sync completed for '${id}': ${lastReport.created_count} created, ${lastReport.updated_count} updated.`;
      if (selectedProfile) {
        activePlan = await previewProfile(selectedProfile.id);
      }
    } catch (e: any) {
      alert('Error applying sync: ' + e);
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

  async function handleExportConfig() {
    try {
      const savedPath = await exportConfigFile();
      if (savedPath) {
        message = `Configuration exported successfully to: ${savedPath}`;
      }
    } catch (err: any) {
      alert('Error exporting configuration: ' + err);
    }
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
  <!-- TOP BAR -->
  <header class="header">
    <div class="logo-area">
      <div class="logo-badge">
        <img src={logoSvg} alt="InSync" class="w-10 h-10" />
      </div>
      <div>
        <div class="flex items-center gap-2">
          <h1>InSync</h1>
          <span class="version-tag">v0.1.0</span>
        </div>
        <p class="subtitle">Controlled, rule-based one-way folder sync</p>
      </div>
    </div>

    <div class="header-actions">
      <!-- Theme Switcher -->
      <button class="btn btn-secondary theme-toggle-btn" on:click={toggleTheme} title="Switch Light / Dark Theme">
        {#if currentTheme === 'dark'}
          <Sun size={15} />
          <span>Light</span>
        {:else}
          <Moon size={15} />
          <span>Dark</span>
        {/if}
      </button>

      <!-- Activity Log -->
      <button class="btn btn-secondary" on:click={showHistory} title="View recent runs">
        <History size={15} />
        <span>Activity Log</span>
      </button>

      <!-- Export Config -->
      <button class="btn btn-secondary" on:click={handleExportConfig} title="Export configuration to file">
        <Download size={15} />
        <span>Export Config</span>
      </button>

      <!-- Diagnostics -->
      <button class="btn btn-secondary" on:click={showDiagnostics} title="View system diagnostics">
        <Activity size={15} />
        <span>Diagnostics</span>
      </button>

      <!-- New Synced Folder -->
      <button class="btn btn-primary" on:click={startCreate}>
        <Plus size={16} strokeWidth={2.5} />
        <span>New Synced Folder</span>
      </button>
    </div>
  </header>

  <!-- ALERT NOTIFICATION -->
  {#if message}
    <div class="alert alert-info">
      <div class="flex items-center gap-2">
        <CheckCircle2 size={16} />
        <span>{message}</span>
      </div>
      <button class="close-btn" on:click={() => (message = '')} title="Dismiss">
        <X size={16} />
      </button>
    </div>
  {/if}

  <div class="content-grid">
    <!-- LEFT PANEL: SYNCED FOLDERS LIST -->
    <section class="card profile-list-card">
      <div class="card-header">
        <div class="flex items-center gap-2">
          <Folder size={18} class="text-[var(--accent-text)]" />
          <h2>Synced Folders</h2>
        </div>
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
                <div class="flex items-center gap-1.5 overflow-hidden text-ellipsis">
                  <span class="path-label">Source:</span>
                  <code title={p.sourceVault}>{p.sourceVault}</code>
                </div>
                <div class="flex items-center gap-1.5 overflow-hidden text-ellipsis">
                  <span class="path-label">Destination:</span>
                  <code title={p.destinationVault}>{p.destinationVault}</code>
                </div>
              </div>
              <div class="rule-summary">
                {#if getRuleCount(p.rules) > 0}
                  <span><strong>Filter Rules:</strong> {getRuleCount(p.rules)} rule{getRuleCount(p.rules) > 1 ? 's' : ''}</span>
                {:else}
                  <span><strong>Filter Rules:</strong> None (all files in scope)</span>
                {/if}
                {#if p.scopes.length > 0}
                  &bull; <span><strong>Scopes:</strong> {p.scopes.join(', ')}</span>
                {/if}
              </div>
            </div>

            <div class="profile-actions">
              <button class="btn btn-outline" on:click={() => handlePreview(p)}>
                <Play size={13} />
                <span>Preview</span>
              </button>
              <button class="btn btn-outline" on:click={() => handleReconcile(p.id)}>
                <FolderSync size={13} />
                <span>Reconcile</span>
              </button>
              <button class="btn btn-outline" on:click={() => startEdit(p)}>
                <Settings size={13} />
                <span>Edit</span>
              </button>
              <button class="btn btn-danger-outline" on:click={() => handleDelete(p.id)}>
                <Trash2 size={13} />
                <span>Delete</span>
              </button>
            </div>
          </div>
        {/each}

        {#if profiles.length === 0}
          <div class="empty-state">
            <Folder size={32} class="mx-auto mb-2 opacity-40 text-[var(--text-muted)]" />
            <p>No synced folders configured yet.</p>
            <button class="btn btn-primary mt-3 inline-flex items-center gap-1.5" on:click={startCreate}>
              <Plus size={15} />
              <span>Add your first synced folder</span>
            </button>
          </div>
        {/if}
      </div>
    </section>

    <!-- RIGHT PANEL: EDIT FORM OR PREVIEW -->
    <section class="card detail-card">
      {#if isEditing}
        <div class="card-header">
          <h2>{editId.startsWith('sync-') ? 'Create Synced Folder' : 'Edit Synced Folder'}</h2>
          <button class="btn btn-ghost" on:click={() => (isEditing = false)}>Cancel</button>
        </div>

        <form on:submit|preventDefault={handleSave}>
          <div class="form-group">
            <label for="name">Folder Sync Name</label>
            <input id="name" bind:value={editName} placeholder="e.g. Work Documents Sync" />
          </div>

          <!-- SOURCE FOLDER WITH NATIVE PICKER -->
          <div class="form-group">
            <label for="source">Source Folder</label>
            <div class="input-picker-row">
              <input id="source" bind:value={editSource} placeholder="/Users/username/SourceFolder" class="flex-1" />
              <button type="button" class="btn btn-secondary picker-btn" on:click={pickSourceFolder} title="Choose folder in Finder">
                <Folder size={15} />
                <span>Browse...</span>
              </button>
            </div>
          </div>

          <!-- DESTINATION FOLDER WITH NATIVE PICKER -->
          <div class="form-group">
            <label for="dest">Destination Folder</label>
            <div class="input-picker-row">
              <input id="dest" bind:value={editDest} placeholder="/Users/username/DestinationFolder" class="flex-1" />
              <button type="button" class="btn btn-secondary picker-btn" on:click={pickDestFolder} title="Choose folder in Finder">
                <Folder size={15} />
                <span>Browse...</span>
              </button>
            </div>
          </div>

          <div class="form-group">
            <label for="scopes">Scope Subfolders (comma-separated)</label>
            <input id="scopes" bind:value={editScopes} placeholder="Courses, Projects/Public" />
            <span class="hint">Leave empty to sync all subfolders in the source folder</span>
          </div>

          <div class="form-group">
            <label for="excludes">Exclusions (comma-separated patterns)</label>
            <input id="excludes" bind:value={editExcludes} placeholder=".git, _private, *.draft.md" />
          </div>

          <!-- MULTI-RULE FILTER BUILDER -->
          <div class="form-box">
            <div class="flex justify-between items-center mb-1">
              <h3>Publishing Filter Rules</h3>
              <button type="button" class="btn btn-sm btn-outline text-xs flex items-center gap-1" on:click={addRule}>
                <Plus size={13} />
                <span>Add Rule</span>
              </button>
            </div>
            <p class="rule-hint">Files are only synced if they match all specified conditions (frontmatter or metadata):</p>

            {#if editRules.length === 0}
              <div class="empty-rules-banner">
                <Info size={16} class="empty-rules-icon shrink-0" />
                <span class="empty-rules-text">
                  No rules active. All files inside the folder scopes will be synced.
                </span>
              </div>
            {:else}
              <div class="flex flex-col gap-2 mt-2">
                {#each editRules as rule, i}
                  <div class="rule-item-row">
                    <div class="flex-1">
                      <input bind:value={rule.property} placeholder="Property (e.g. status)" class="w-full text-xs" />
                    </div>
                    <div class="w-32">
                      <select bind:value={rule.operator} class="w-full text-xs">
                        <option value="equals">equals</option>
                        <option value="notEquals">not equals</option>
                        <option value="contains">contains</option>
                        <option value="exists">exists</option>
                        <option value="notExists">not exists</option>
                        <option value="in">in</option>
                      </select>
                    </div>
                    {#if rule.operator !== 'exists' && rule.operator !== 'notExists'}
                      <div class="flex-1">
                        <input bind:value={rule.value} placeholder="Value (e.g. completed)" class="w-full text-xs" />
                      </div>
                    {/if}
                    <button type="button" class="btn-icon-danger" on:click={() => removeRule(i)} title="Remove this rule">
                      <Trash2 size={14} />
                    </button>
                  </div>
                {/each}
              </div>
            {/if}
          </div>

          <div class="form-group">
            <label for="att-mode" class="flex items-center justify-between">
              <span>File Sync & Asset Strategy</span>
              <span class="text-[0.72rem] text-[var(--accent-text)] font-normal">Controls non-markdown media</span>
            </label>
            <div class="select-wrapper">
              <select id="att-mode" bind:value={editAttachmentMode}>
                <option value="explicit">All files & assets (Default — syncs all files in folder)</option>
                <option value="referenced">Referenced only (Only copy images and assets linked in notes)</option>
                <option value="markdown-only">Markdown only (Exclude all non-markdown files)</option>
              </select>
            </div>
            <span class="hint">Choose whether all files or only markdown-referenced assets are synced to the destination.</span>
          </div>

          <div class="form-actions">
            <button type="button" class="btn btn-secondary" on:click={() => (isEditing = false)}>Cancel</button>
            <button type="submit" class="btn btn-primary">Save Synced Folder</button>
          </div>
        </form>
      {:else if isPreviewing && activePlan}
        <div class="card-header">
          <div>
            <h2>Sync Preview: {selectedProfile?.name}</h2>
            <span class="subtitle">{activePlan.actions.length} planned operations</span>
          </div>
          <div style="display: flex; gap: 8px;">
            <button class="btn btn-primary" on:click={() => activePlan && handleApply(activePlan.profile_id)}>
              <Play size={14} />
              <span>Apply Sync Now</span>
            </button>
            <button class="btn btn-ghost" on:click={() => (isPreviewing = false)}>Close</button>
          </div>
        </div>

        {#if lastReport}
          <div class="report-banner">
            <div class="font-semibold mb-1">Execution Summary:</div>
            +{lastReport.created_count} created, ~{lastReport.updated_count} updated,
            -{lastReport.archived_count} archived, {lastReport.skipped_count} skipped
            {#if lastReport.conflict_count > 0}
              <span class="text-danger font-semibold">&bull; {lastReport.conflict_count} external collaborator conflicts preserved</span>
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
              <CheckCircle2 size={24} class="mx-auto mb-1 text-nord14 opacity-80" />
              <p>Everything is currently up to date. No pending sync actions.</p>
            </div>
          {/if}
        </div>
      {:else}
        <div class="placeholder-state">
          <FolderSync size={44} class="placeholder-icon" />
          <h3>Select a Synced Folder</h3>
          <p>Click "Preview" on any synced folder to inspect changes before writing files, or click "+ New Synced Folder" to configure a new folder mapping.</p>
        </div>
      {/if}
    </section>
  </div>

  <!-- DIAGNOSTICS MODAL -->
  {#if diagnosticsData}
    <div class="modal-backdrop" on:click={() => (diagnosticsData = null)} on:keydown={(e) => e.key === 'Escape' && (diagnosticsData = null)} role="presentation">
      <div class="modal" role="dialog" aria-modal="true" tabindex="-1" on:click|stopPropagation on:keydown|stopPropagation>
        <div class="flex justify-between items-center mb-3">
          <h2>System Diagnostics</h2>
          <button class="btn-icon" on:click={() => (diagnosticsData = null)}><X size={16} /></button>
        </div>
        <pre>{JSON.stringify(diagnosticsData, null, 2)}</pre>
        <div class="flex justify-end mt-4">
          <button class="btn btn-secondary" on:click={() => (diagnosticsData = null)}>Close</button>
        </div>
      </div>
    </div>
  {/if}

  <!-- HISTORY MODAL -->
  {#if isViewingHistory}
    <div class="modal-backdrop" on:click={() => (isViewingHistory = false)} on:keydown={(e) => e.key === 'Escape' && (isViewingHistory = false)} role="presentation">
      <div class="modal" role="dialog" aria-modal="true" tabindex="-1" style="max-width: 680px;" on:click|stopPropagation on:keydown|stopPropagation>
        <div class="flex justify-between items-center mb-3">
          <h2>Recent Execution History</h2>
          <button class="btn-icon" on:click={() => (isViewingHistory = false)}><X size={16} /></button>
        </div>
        {#if historyList.length === 0}
          <p style="color: var(--text-muted); margin: 20px 0;">No run history entries recorded yet.</p>
        {:else}
          <div style="max-height: 380px; overflow-y: auto; margin-bottom: 16px;">
            <table style="width: 100%; border-collapse: collapse; font-size: 0.85rem; color: var(--text-primary);">
              <thead>
                <tr style="border-bottom: 2px solid var(--border-color); text-align: left; color: var(--text-muted);">
                  <th style="padding: 8px 6px;">Folder Sync</th>
                  <th style="padding: 8px 6px;">Created</th>
                  <th style="padding: 8px 6px;">Updated</th>
                  <th style="padding: 8px 6px;">Archived</th>
                  <th style="padding: 8px 6px;">Conflicts</th>
                  <th style="padding: 8px 6px;">Duration</th>
                </tr>
              </thead>
              <tbody>
                {#each historyList as h}
                  <tr style="border-bottom: 1px solid var(--border-color);">
                    <td style="padding: 8px 6px;"><strong>{h.profileId}</strong></td>
                    <td style="padding: 8px 6px; color: var(--badge-create-text);">+{h.created}</td>
                    <td style="padding: 8px 6px; color: var(--badge-update-text);">~{h.updated}</td>
                    <td style="padding: 8px 6px; color: var(--badge-archive-text);">-{h.archived}</td>
                    <td style="padding: 8px 6px; color: var(--badge-conflict-text);">{h.conflicts}</td>
                    <td style="padding: 8px 6px; color: var(--text-muted);">{h.durationMs}ms</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
        <div class="flex justify-end">
          <button class="btn btn-secondary" on:click={() => (isViewingHistory = false)}>Close</button>
        </div>
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
    padding-bottom: 18px;
    border-bottom: 1px solid var(--border-color);
  }

  .logo-area {
    display: flex;
    align-items: center;
    gap: 14px;
  }

  .logo-badge {
    width: 42px;
    height: 42px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .logo-badge img {
    width: 100%;
    height: 100%;
    object-fit: contain;
  }

  h1 {
    margin: 0;
    font-size: 1.45rem;
    font-weight: 700;
    letter-spacing: -0.02em;
    color: var(--text-h);
  }

  .version-tag {
    font-size: 0.7rem;
    font-weight: 600;
    padding: 1px 6px;
    border-radius: 4px;
    background: var(--btn-sec-bg);
    color: var(--text-muted);
    border: 1px solid var(--border-color);
  }

  .subtitle {
    margin: 2px 0 0;
    color: var(--text-muted);
    font-size: 0.85rem;
  }

  .header-actions {
    display: flex;
    gap: 8px;
    align-items: center;
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

  /* Sleek Cards */
  .card {
    background: var(--bg-card);
    border-radius: 10px;
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
    font-size: 1.15rem;
    font-weight: 600;
    letter-spacing: -0.01em;
    color: var(--text-h);
  }

  .count-tag {
    background: var(--accent-subtle);
    color: var(--accent-text);
    padding: 2px 10px;
    border-radius: 12px;
    font-size: 0.78rem;
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
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-h);
  }

  .badge-active {
    background: var(--badge-create-bg);
    color: var(--badge-create-text);
    padding: 2px 8px;
    border-radius: 10px;
    font-size: 0.72rem;
    font-weight: 600;
  }

  .path-details {
    font-size: 0.82rem;
    margin: 6px 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .path-label {
    color: var(--text-muted);
    font-weight: 500;
    font-size: 0.78rem;
    min-width: 75px;
  }

  code {
    background: var(--code-bg);
    color: var(--code-text);
    padding: 2px 6px;
    border-radius: 4px;
    font-family: var(--font-mono);
    font-size: 0.78rem;
    border: 1px solid var(--border-color);
  }

  .rule-summary {
    font-size: 0.8rem;
    color: var(--text-secondary);
  }

  .profile-actions {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }

  /* Sleek Buttons (shadcn style) */
  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 7px 13px;
    border-radius: 6px;
    font-size: 0.82rem;
    font-weight: 500;
    cursor: pointer;
    border: 1px solid transparent;
    transition: all 0.15s ease;
  }

  .btn-sm {
    padding: 4px 8px;
    font-size: 0.75rem;
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
    color: var(--badge-conflict-text);
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

  .btn-icon {
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 4px;
    border-radius: 4px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }
  .btn-icon:hover {
    color: var(--text-primary);
    background: var(--btn-sec-bg);
  }

  .btn-icon-danger {
    background: transparent;
    border: 1px solid transparent;
    color: var(--badge-conflict-text);
    cursor: pointer;
    padding: 6px;
    border-radius: 6px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s ease;
  }
  .btn-icon-danger:hover {
    background: var(--badge-conflict-bg);
  }

  /* Form Elements */
  .form-group {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-bottom: 14px;
  }

  .form-group label {
    font-size: 0.82rem;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .input-picker-row {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .picker-btn {
    white-space: nowrap;
    padding: 8px 12px;
  }

  input {
    padding: 7px 11px;
    background: var(--input-bg);
    color: var(--input-text);
    border: 1px solid var(--input-border);
    border-radius: 6px;
    font-size: 0.85rem;
    font-family: inherit;
    transition: border-color 0.15s ease, box-shadow 0.15s ease;
  }

  input:focus {
    outline: none;
    border-color: var(--input-focus);
    box-shadow: 0 0 0 2px var(--accent-subtle);
  }

  .select-wrapper {
    position: relative;
    display: flex;
    align-items: center;
    width: 100%;
  }

  select {
    appearance: none;
    -webkit-appearance: none;
    -moz-appearance: none;
    width: 100%;
    padding: 8px 34px 8px 12px;
    background-color: var(--input-bg);
    color: var(--input-text);
    border: 1px solid var(--input-border);
    border-radius: 6px;
    font-size: 0.85rem;
    font-family: inherit;
    line-height: 1.4;
    cursor: pointer;
    background-image: url("data:image/svg+xml;charset=UTF-8,%3csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='%235e81ac' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3e%3cpolyline points='6 9 12 15 18 9'%3e%3c/polyline%3e%3c/svg%3e");
    background-repeat: no-repeat;
    background-position: right 12px center;
    background-size: 14px 14px;
    transition: border-color 0.15s ease, box-shadow 0.15s ease;
  }

  :global([data-theme="dark"]) select {
    background-image: url("data:image/svg+xml;charset=UTF-8,%3csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='%2388c0d0' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3e%3cpolyline points='6 9 12 15 18 9'%3e%3c/polyline%3e%3c/svg%3e");
  }

  select:hover {
    border-color: var(--border-active);
  }

  select:focus {
    outline: none;
    border-color: var(--input-focus);
    box-shadow: 0 0 0 2px var(--accent-subtle);
  }

  .empty-rules-banner {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 14px;
    margin-top: 8px;
    border-radius: 6px;
    background: var(--accent-subtle);
    border: 1px dashed var(--border-active);
  }

  .empty-rules-text {
    font-size: 0.82rem;
    font-weight: 500;
    color: var(--text-primary);
    line-height: 1.4;
  }

  .empty-rules-icon {
    color: var(--accent-text);
  }

  .hint {
    font-size: 0.74rem;
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
    margin: 0;
    font-size: 0.9rem;
    font-weight: 600;
    color: var(--text-h);
  }

  .rule-hint {
    margin: 2px 0 8px;
    font-size: 0.76rem;
    color: var(--text-muted);
  }

  .rule-item-row {
    display: flex;
    gap: 6px;
    align-items: center;
  }

  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 16px;
  }

  /* Plan Rows & Diffs */
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
    font-size: 0.82rem;
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
    font-size: 0.72rem;
    padding: 2px 8px;
    border-radius: 4px;
    font-weight: 600;
    min-width: 60px;
    text-align: center;
  }

  .action-path {
    font-family: var(--font-mono);
    flex: 1;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .action-reason {
    font-size: 0.74rem;
    color: var(--badge-conflict-text);
  }

  .report-banner {
    background: var(--badge-create-bg);
    border: 1px solid var(--border-color);
    color: var(--badge-create-text);
    padding: 10px 14px;
    border-radius: 6px;
    margin-bottom: 14px;
    font-size: 0.82rem;
  }

  .text-danger { color: var(--badge-conflict-text); }

  .placeholder-state {
    text-align: center;
    padding: 60px 20px;
    color: var(--text-muted);
  }

  .placeholder-state h3 {
    color: var(--text-h);
    margin-top: 12px;
    font-size: 1.1rem;
  }

  .placeholder-icon {
    margin: 0 auto;
    opacity: 0.4;
    color: var(--accent);
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
    font-size: 0.85rem;
  }

  .alert-info {
    background: var(--badge-update-bg);
    border: 1px solid var(--border-color);
    color: var(--badge-update-text);
  }

  .close-btn {
    background: none;
    border: none;
    cursor: pointer;
    color: inherit;
    display: inline-flex;
    align-items: center;
    opacity: 0.8;
  }
  .close-btn:hover {
    opacity: 1;
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
    border-radius: 10px;
    max-width: 520px;
    width: 100%;
    border: 1px solid var(--border-color);
    box-shadow: var(--shadow);
  }

  .modal h2 {
    color: var(--text-h);
    margin: 0;
    font-size: 1.15rem;
  }

  pre {
    background: var(--code-bg);
    color: var(--code-text);
    border: 1px solid var(--border-color);
    padding: 12px;
    border-radius: 6px;
    overflow-x: auto;
    font-size: 0.78rem;
    font-family: var(--font-mono);
  }
</style>
