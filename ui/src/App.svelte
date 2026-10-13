<script lang="ts">
  import { onMount } from 'svelte';
  import type { ProfileConfig, Plan, RunReport, RulePredicate, AttachmentMode } from './types';
  import {
    listProfiles,
    saveProfile,
    deleteProfile,
    previewProfile,
    applyProfile,
    reconcileProfile,
    getDiagnostics,
  } from './api';

  let profiles: ProfileConfig[] = [];
  let selectedProfile: ProfileConfig | null = null;
  let isEditing = false;
  let isPreviewing = false;
  let activePlan: Plan | null = null;
  let lastReport: RunReport | null = null;
  let diagnosticsData: any = null;
  let loading = false;
  let message = '';

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
    await refreshProfiles();
  });

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
      <button class="btn btn-secondary" on:click={exportConfig}>Export Config</button>
      <button class="btn btn-secondary" on:click={showDiagnostics}>Diagnostics</button>
      <button class="btn btn-primary" on:click={startCreate}>+ New Profile</button>
    </div>
  </header>

  {#if message}
    <div class="alert alert-info">
      {message}
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
      </div>
    </section>

    <!-- RIGHT PANE: EDITOR OR PREVIEW -->
    <section class="card detail-card">
      {#if isEditing}
        <div class="card-header">
          <h2>{editId ? 'Edit Profile' : 'New Profile'}</h2>
          <button class="btn btn-ghost" on:click={() => (isEditing = false)}>Cancel</button>
        </div>

        <form class="editor-form" on:submit|preventDefault={handleSave}>
          <div class="form-group">
            <label for="p-name">Profile Name</label>
            <input id="p-name" type="text" bind:value={editName} placeholder="e.g. Shared Courses" required />
          </div>

          <div class="form-row">
            <div class="form-group flex-1">
              <label for="p-src">Source Vault Directory</label>
              <input id="p-src" type="text" bind:value={editSource} placeholder="/Users/name/Vaults/Private" required />
            </div>
            <div class="form-group flex-1">
              <label for="p-dst">Destination Vault Directory</label>
              <input id="p-dst" type="text" bind:value={editDest} placeholder="/Users/name/Shared/CourseVault" required />
            </div>
          </div>

          <div class="form-group">
            <label for="p-scopes">Included Scopes (Folders)</label>
            <input id="p-scopes" type="text" bind:value={editScopes} placeholder="Courses, Projects/Public" />
            <small class="hint">Only files within these source subfolders will be evaluated.</small>
          </div>

          <div class="form-group">
            <label for="p-excludes">Exclusions (Patterns / Folders)</label>
            <input id="p-excludes" type="text" bind:value={editExcludes} placeholder=".obsidian, _private, *.draft.md" />
          </div>

          <div class="form-box">
            <h3>Publishing Rule Filter</h3>
            <p class="rule-hint">Notes publish when frontmatter matches:</p>
            <div class="form-row">
              <input type="text" bind:value={editRuleProp} placeholder="Property (e.g. status)" style="width: 140px;" />
              <select bind:value={editRuleOp} style="width: 130px;">
                <option value="equals">equals</option>
                <option value="notEquals">notEquals</option>
                <option value="exists">exists</option>
                <option value="notExists">notExists</option>
                <option value="contains">contains</option>
                <option value="in">in</option>
              </select>
              <input type="text" bind:value={editRuleVal} placeholder="Value (e.g. completed)" class="flex-1" />
            </div>
          </div>

          <div class="form-group">
            <label for="p-att">Attachment Policy</label>
            <select id="p-att" bind:value={editAttachmentMode}>
              <option value="referenced">Referenced Attachments (Images/PDFs linked in notes)</option>
              <option value="markdown-only">Markdown Only (Do not copy attachments)</option>
              <option value="explicit">Explicit Eligibility</option>
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
            <p class="subtitle">{activePlan.actions.length} planned operations</p>
          </div>
          <div class="header-actions">
            <button class="btn btn-primary" on:click={() => handleApply(activePlan.profile_id)}>
              Apply Plan Now
            </button>
            <button class="btn btn-ghost" on:click={() => (isPreviewing = false)}>Close</button>
          </div>
        </div>

        {#if lastReport}
          <div class="report-banner">
            <strong>Execution Completed:</strong>
            +{lastReport.created_count} Created,
            ~{lastReport.updated_count} Updated,
            -{lastReport.archived_count} Archived,
            {lastReport.skipped_count} Skipped,
            <span class="text-danger">{lastReport.conflict_count} Conflicts</span>
          </div>
        {/if}

        <div class="plan-actions-list">
          {#if activePlan.actions.length === 0}
            <div class="empty-state">No matching files or changes needed. All in sync.</div>
          {:else}
            {#each activePlan.actions as action}
              {@const key = getActionKey(action)}
              {@const path = getActionPath(action)}
              {@const reason = getActionReason(action)}
              <div class="plan-row {key.toLowerCase()}">
                <span class="action-tag tag-{key.toLowerCase()}">{key}</span>
                <span class="action-path">{path}</span>
                {#if reason}
                  <span class="action-reason">{reason}</span>
                {/if}
              </div>
            {/each}
          {/if}
        </div>

      {:else}
        <div class="placeholder-state">
          <div class="placeholder-icon">📋</div>
          <h3>Select a Profile</h3>
          <p>Click <strong>Preview</strong> on any profile to inspect planned operations, or <strong>Edit</strong> to configure rules.</p>
        </div>
      {/if}
    </section>
  </div>

  {#if diagnosticsData}
    <div class="modal-backdrop" role="presentation" on:click={() => (diagnosticsData = null)} on:keydown={(e) => e.key === 'Escape' && (diagnosticsData = null)}>
      <div class="modal" role="dialog" aria-modal="true" on:click|stopPropagation on:keydown|stopPropagation>
        <h2>Diagnostics Report</h2>
        <pre>{JSON.stringify(diagnosticsData, null, 2)}</pre>
        <button class="btn btn-secondary" on:click={() => (diagnosticsData = null)}>Close</button>
      </div>
    </div>
  {/if}
</main>

<style>
  :global(body) {
    margin: 0;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    background-color: #f8fafc;
    color: #1e293b;
  }

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
    border-bottom: 1px solid #e2e8f0;
  }

  .logo-area {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .logo-badge {
    width: 44px;
    height: 44px;
    background: #4f46e5;
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
  }

  .subtitle {
    margin: 4px 0 0;
    color: #64748b;
    font-size: 0.9rem;
  }

  .header-actions {
    display: flex;
    gap: 8px;
  }

  .content-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 20px;
  }

  .card {
    background: white;
    border-radius: 12px;
    border: 1px solid #e2e8f0;
    padding: 20px;
    box-shadow: 0 1px 3px rgba(0,0,0,0.05);
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
  }

  .count-tag {
    background: #e0e7ff;
    color: #4338ca;
    padding: 2px 8px;
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
    border: 1px solid #e2e8f0;
    border-radius: 8px;
    padding: 14px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    transition: border-color 0.15s;
  }

  .profile-item.active {
    border-color: #6366f1;
    background-color: #f5f7ff;
  }

  .profile-title-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .profile-title-row h3 {
    margin: 0;
    font-size: 1.05rem;
  }

  .badge-active {
    background: #dcfce7;
    color: #15803d;
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
    gap: 2px;
  }

  .path-label {
    color: #64748b;
    font-weight: 500;
  }

  code {
    background: #f1f5f9;
    padding: 2px 4px;
    border-radius: 4px;
    font-family: monospace;
    font-size: 0.8rem;
  }

  .rule-summary {
    font-size: 0.85rem;
    color: #475569;
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
  }

  .btn-primary { background: #4f46e5; color: white; }
  .btn-primary:hover { background: #4338ca; }

  .btn-secondary { background: #f1f5f9; color: #334155; border-color: #cbd5e1; }
  .btn-secondary:hover { background: #e2e8f0; }

  .btn-outline { background: white; border-color: #cbd5e1; color: #334155; }
  .btn-outline:hover { background: #f8fafc; }

  .btn-danger-outline { background: white; border-color: #fca5a5; color: #b91c1c; }
  .btn-danger-outline:hover { background: #fee2e2; }

  .btn-ghost { background: transparent; color: #64748b; }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-bottom: 14px;
  }

  .form-group label {
    font-size: 0.85rem;
    font-weight: 600;
  }

  input, select {
    padding: 8px 10px;
    border: 1px solid #cbd5e1;
    border-radius: 6px;
    font-size: 0.9rem;
  }

  .form-row {
    display: flex;
    gap: 10px;
    align-items: center;
  }

  .flex-1 { flex: 1; }

  .hint {
    font-size: 0.75rem;
    color: #64748b;
  }

  .form-box {
    background: #f8fafc;
    border: 1px solid #e2e8f0;
    padding: 12px;
    border-radius: 8px;
    margin-bottom: 14px;
  }

  .form-box h3 {
    margin: 0 0 4px;
    font-size: 0.95rem;
  }

  .rule-hint {
    margin: 0 0 10px;
    font-size: 0.8rem;
    color: #64748b;
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
    background: #f8fafc;
    border: 1px solid #e2e8f0;
  }

  .tag-create { background: #dcfce7; color: #15803d; }
  .tag-update { background: #e0f2fe; color: #0369a1; }
  .tag-skip { background: #f1f5f9; color: #64748b; }
  .tag-archive { background: #fef3c7; color: #b45309; }
  .tag-conflict { background: #fee2e2; color: #b91c1c; font-weight: 700; }
  .tag-warn { background: #ffedd5; color: #c2410c; }

  .action-tag {
    font-size: 0.75rem;
    padding: 2px 6px;
    border-radius: 4px;
    font-weight: 600;
    min-width: 60px;
    text-align: center;
  }

  .action-path {
    font-family: monospace;
    flex: 1;
  }

  .action-reason {
    font-size: 0.75rem;
    color: #dc2626;
  }

  .report-banner {
    background: #ecfdf5;
    border: 1px solid #a7f3d0;
    color: #065f46;
    padding: 10px 14px;
    border-radius: 6px;
    margin-bottom: 14px;
    font-size: 0.85rem;
  }

  .text-danger { color: #dc2626; font-weight: 600; }

  .placeholder-state {
    text-align: center;
    padding: 60px 20px;
    color: #64748b;
  }

  .placeholder-icon {
    font-size: 2.5rem;
    margin-bottom: 12px;
  }

  .empty-state {
    text-align: center;
    padding: 30px;
    color: #64748b;
  }

  .alert {
    padding: 10px 14px;
    border-radius: 6px;
    margin-bottom: 16px;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .alert-info { background: #eff6ff; border: 1px solid #bfdbfe; color: #1e40af; }
  .close-btn { background: none; border: none; font-size: 1.2rem; cursor: pointer; color: inherit; }

  .modal-backdrop {
    position: fixed;
    top: 0; left: 0; right: 0; bottom: 0;
    background: rgba(0,0,0,0.4);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .modal {
    background: white;
    padding: 24px;
    border-radius: 12px;
    max-width: 500px;
    width: 100%;
  }

  pre {
    background: #f1f5f9;
    padding: 12px;
    border-radius: 6px;
    overflow-x: auto;
    font-size: 0.8rem;
  }
</style>
