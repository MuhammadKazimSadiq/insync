# Spec: Vault Publisher

Status: ready-for-agent

## Problem Statement

Knowledge workers, researchers, educators, and consultants maintain extensive notes in private Obsidian vaults containing a mixture of confidential personal logs, early drafts, sensitive client data, and finished publishable work. Currently, when users wish to share specific completed notes or documentation with teams, clients, or study groups via shared folders (OneDrive, Google Drive, Dropbox, Syncthing), they are forced to either:
1. Maintain separate duplicate vaults, creating friction, manual copy-pasting, and link rot; or
2. Share their entire vault or an entire folder, creating catastrophic privacy risks of exposing unready notes, private tags, or confidential attachments.

Existing sync utilities operate as indiscriminate file synchronizers without semantic awareness of note metadata. They lack the ability to inspect Markdown frontmatter properties, cannot distinguish between public and private notes in the same folder, and risk overwriting private source files when collaborators edit shared files.

## Solution

Vault Publisher is a cross-platform desktop application and background agent that functions as a deterministic, one-way rule-based publisher. It projects a curated subset of notes and attachments from an authoritative private source vault into a shared destination vault based on user-defined profiles, folder scopes, and YAML frontmatter metadata rules (e.g., `status: completed` AND `share: true`).

The solution guarantees safety and data preservation:
- It generates a detailed preview plan (`Create`, `Update`, `Archive`, `Skip`, `Warn`, `Conflict`) before mutating destination storage.
- It tracks published files via a versioned manifest ledger (`.vault-publisher/manifest.json`) in the destination vault.
- It never deletes unmanaged files in the destination vault.
- It moves unpublishing notes into an archive directory rather than permanently deleting them.
- It detects collaborator edits in the destination vault, flagging conflicts and preserving both copies without overwriting.
- It keeps the projection continuously synchronized using debounced filesystem events combined with periodic reconciliation passes to handle system sleep and cloud provider latencies.
- It runs as an independent headless background agent with tray status, managed through a clean Tauri desktop interface.

## User Stories

1. As a note author, I want to define one or more publishing profiles, so that I can publish different subsets of notes to different destinations (e.g., public course notes vs. team project docs).
2. As a note author, I want to select a private source vault and a shared destination vault for each profile, so that the engine knows where to read original notes and where to project published files.
3. As a note author, I want the system to reject configurations where source and destination paths are identical or where the destination is nested inside the source, so that recursive publishing loops and accidental data destruction are prevented.
4. As a note author, I want to define one or more scoped source folders per profile, so that the publisher only inspects directories I explicitly permit.
5. As a note author, I want to configure path exclusion patterns (e.g., `.obsidian`, `_private`, `**/*.draft.md`), so that internal metadata, credentials, and private subtrees are never scanned or published.
6. As a note author, I want to define metadata rules matching YAML frontmatter fields (such as `status equals completed` or `share equals true`), so that only notes meeting explicit readiness criteria are published.
7. As a note author, I want support for common rule operators (`equals`, `notEquals`, `exists`, `notExists`, `contains`, `in`), so that I can express diverse publishing logic against strings, booleans, numbers, dates, and tag arrays.
8. As a note author, I want to combine rules with logical `all` (AND) and `any` (OR) groups, so that I can express compound eligibility conditions.
9. As a note author, I want strict type handling in rule evaluations (e.g., boolean `true` is never conflated with string `"true"`), so that accidental false-positive publications do not occur.
10. As a note author, I want the publisher to preserve relative directory paths from the source scope into the destination vault, so that folder hierarchies remain intuitive and organized for collaborators.
11. As a note author, I want to configure an attachment policy (`markdown-only`, `referenced`, or `explicit`), so that I control how non-Markdown media (images, PDFs) are shared.
12. As a note author, I want referenced attachments to be discovered from Markdown wikilinks and standard Markdown image/file links, so that published notes do not display broken image icons.
13. As a note author, I want missing attachment references to generate warnings rather than failing the entire publishing run, so that minor attachment omissions do not block publishing valid notes.
14. As a note author, I want a preview mode that shows every planned action before execution, so that I can inspect what files will be created, updated, archived, or skipped.
15. As a note author, I want a dry-run flag for manual and automated invocations, so that I can verify publishing outcomes without touching the destination filesystem.
16. As a note author, I want the publisher to execute file copies atomically using temporary staging files and atomic replacement, so that half-written files are never exposed to cloud sync services or collaborators.
17. As a note author, I want the destination state to be tracked in a manifest ledger (`.vault-publisher/manifest.json`), so that the system knows exactly which files are managed by each profile.
18. As a note author, I want the system to never delete or alter destination files not tracked in the manifest ledger, so that collaborator-authored files placed in the shared vault remain untouched.
19. As a note author, I want notes that no longer match publishing rules (e.g., status changed back to `draft`) to be moved to a designated archive folder rather than deleted, so that collaborator comments and historical context are not lost.
20. As a note author, I want collaborator edits on managed destination notes to be flagged as conflicts, so that external changes are never silently overwritten by source updates.
21. As a note author, I want event-driven file watching on source scopes with debouncing, so that saving edits in Obsidian triggers prompt publication without thrashing on multiple keystrokes.
22. As a note author, I want periodic reconciliation runs at configurable intervals (e.g., every 15 minutes), so that changes missed during laptop sleep, offline periods, or cloud provider synchronization delays are automatically caught and corrected.
23. As a note author, I want malformed frontmatter in an individual note to be logged as a warning and skipped, so that a single syntax mistake does not abort publication of other valid notes.
24. As a note author, I want the publishing engine to run as a headless background agent with a single-instance lock, so that scheduled publishing continues silently even when the graphical window is closed.
25. As a note author, I want a desktop interface to create, edit, validate, and test publishing profiles, so that I don't have to write raw JSON configuration manually.
26. As a note author, I want plain-language rule summaries in the user interface (e.g., "Publish notes in Courses when status equals completed"), so that rule logic is easy to audit at a glance.
27. As a note author, I want to trigger a manual "Run Now" or "Reconcile Now" action from the desktop UI or system tray, so that I can immediately push urgent updates.
28. As a note author, I want to view run histories and execution logs with filtered outcomes (created, updated, archived, conflicts, warnings), so that I can troubleshoot publishing behavior.
29. As a note author, I want to pause and resume profiles, so that I can make bulk edits in my private vault without triggering intermediate publishing runs.
30. As a note author, I want to export and import profile configurations, so that I can backup or replicate configurations across machines.
31. As a note author, I want an exportable diagnostics report capturing system state, profile summaries, and recent error logs, so that I can easily diagnose operational failures.

## Implementation Decisions

### Engine and Shell Boundary
The publishing engine is structured as an independent core module completely isolated from windowing, graphics, and desktop framework dependencies. The desktop shell acts purely as an ambient configuration, visualization, and command adapter around the core engine.

```
+-------------------------------------------------------+
|                     Tauri 2 Shell                     |
|  (Svelte UI, System Tray, OS Notifications, Settings) |
+-------------------------------------------------------+
                           |
                     IPC Commands
                           |
+-------------------------------------------------------+
|                    Publisher Core                     |
|  (Config, Metadata, Rules, Scanner, Planner, Engine)  |
+-------------------------------------------------------+
                           |
             Headless CLI / Daemon Runner
```

### Core Engine Interface
The core engine exposes a deep lifecycle API operating on structured requests:
- `preview(request)`: Evaluates configuration against source and destination vaults to generate a deterministic, immutable `Plan` without mutating storage.
- `apply(plan)`: Executes the calculated operations atomically, records manifest state, and returns a detailed execution report.
- `reconcile(request)`: Executes a full-scan projection recalculation, repairs state drifts, audits destination integrity, and commits changes.

### Plan Model
A plan consists of discrete typed actions:
- `Create`: Destination note does not exist; copy source note atomically.
- `Update`: Destination note exists, matches previous manifest hash, and source has changed; replace atomically.
- `Archive`: Note was previously published and recorded in manifest, but no longer matches rules or was deleted from source; move to destination archive path.
- `Skip`: Source note matches rules but content hash is identical to destination manifest; no filesystem mutation needed.
- `Warn`: Malformed frontmatter, unresolvable referenced attachment, or non-fatal permission limitation; note skipped or published without attachment.
- `Conflict`: Destination file content hash differs from manifest recorded hash, indicating external modification; preserve destination file and abort overwrite for this note.

### Rule Evaluation Model
The rule engine evaluates structured predicates against extracted frontmatter:
- Supports operators: `equals`, `notEquals`, `exists`, `notExists`, `contains`, `in`.
- Supports recursive `all` (logical AND) and `any` (logical OR) boolean groups.
- Strict type semantics: values retain exact YAML types (string, boolean, integer, float, date, list of strings). Type coercion is forbidden.
- Frontmatter parsing extracts only the initial YAML block delimited by `---` lines.

### File Mutation Safety
- Atomic Writes: Destination updates write to a hidden sibling temporary file (`.tmp.vault-publisher-*`) before atomic rename/replace.
- Non-destructive archiving: Default retention moves dropped files into `.vault-publisher-archive/<timestamp>/` rather than deleting.
- Unmanaged file protection: Files present in destination but absent from `.vault-publisher/manifest.json` are strictly unmanaged and will never be deleted, moved, or overwritten.

### Destination Manifest Ledger
The destination vault contains a versioned JSON manifest located at `.vault-publisher/manifest.json` containing:
- Engine schema version
- Profile ID
- Map of destination paths to records containing: source relative path, SHA-256 source content hash, SHA-256 destination content hash, published timestamp, rule snapshot, and attachment dependencies.

### Dual-Paced Scheduling Engine
The publishing scheduler implements a dual-paced synchronization architecture:
- Event-Driven Watcher: Filesystem notifications on configured scopes are debounced with a configurable quiet window (e.g., 2–5 seconds) to avoid thrashing during active editing.
- Periodic Reconciliation: A background timer executes full reconciliation at configured intervals (e.g., every 15 minutes) to ensure consistency even if filesystem notifications were dropped during OS sleep or cloud sync latency.
- Single-Instance Daemon Lock: Background execution acquires an OS file lock on profile execution to prevent concurrent publishing runs.

### Desktop UI Adapter
The desktop interface is built using Svelte and TypeScript communicating with Tauri 2 IPC commands. The UI provides:
- Profile management (list, create, edit, delete, duplicate)
- Interactive visual rule builder generating validated profile configurations
- Plan preview modal with side-by-side status indicators before manual apply
- Real-time activity log and run history viewer
- System tray integration with pause/resume controls and status badges

## Testing Decisions

### What Makes a Good Test
Tests must verify external behavioral contracts against real temporary filesystem directories rather than mocking internal functions. A good test creates concrete source vault files, runs engine operations, and verifies exact destination filesystem trees, file contents, and manifest ledger entries.

### Testing Seams
1. **Engine Public API Seam (Primary)**: Tests instantiate the core publishing engine against temporary directories created via `tempfile`, validating `preview`, `apply`, and `reconcile` deterministically without Tauri or UI dependencies.
2. **Headless CLI Seam (Secondary)**: Integration tests invoke the compiled CLI binary with configuration files and arguments, verifying exit codes, stdout plan outputs, and filesystem side-effects.
3. **IPC Command Seam (Desktop)**: Tauri command handler tests verify serialization, error mapping, and asynchronous progress event emission.

### Test Categories
- **Metadata Parsing**: Valid YAML frontmatter, missing frontmatter, malformed YAML handling, type fidelity (boolean vs. string, numbers, dates), frontmatter tag arrays, and Unicode paths.
- **Rule Evaluation**: All operator combinations, nested boolean groups, missing property behavior, and type mismatch rejections.
- **Planner & Executor**: Create, update, skip, archive transitions; atomic temporary file replacement; directory hierarchy mirroring; symlink and directory escape rejection.
- **Safety & Conflicts**: Preservation of unmanaged destination files, conflict generation when destination file is modified externally, and non-destructive archiving.
- **Reconciliation & Idempotency**: Running reconciliation repeatedly on unchanged vaults produces zero file mutations; recovering from deleted manifest entries; healing missed source deletions.

### Prior Art
Testing patterns follow headless compiler architectures, Git worktree operations, and Terraform-style declarative plan/apply integration suites.

## Out of Scope

- Two-way bi-directional synchronization or merging external destination edits back into the source vault.
- Real-time collaborative multi-user editing.
- Cloud storage backend hosting, user accounts, or proprietary relay servers.
- Mobile platforms (iOS and Android).
- Arbitrary code execution or scripting languages embedded within rule definitions.

## Further Notes

The development roadmap adheres strictly to a CLI-first, engine-first milestone:
1. Proof-of-concept headless engine with full temporary directory test coverage and CLI runner.
2. Tauri desktop application shell with configuration editor, preview screen, and activity log.
3. Background watching daemon with debounced events, scheduled reconciliation, and system tray status.
4. Packaging, platform startup registration (macOS launch agent, Windows startup), and installers.
