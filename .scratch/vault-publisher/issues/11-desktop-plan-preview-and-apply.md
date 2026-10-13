# 11: Desktop Plan Preview and Interactive Apply

**What to build:**
An interactive Plan Preview and Execution screen in the Tauri desktop application. The user can click "Preview" on any profile to trigger the engine's `preview` command, displaying a clear breakdown of planned changes (`Create`, `Update`, `Archive`, `Skip`, `Conflict`, `Warn`) with file paths and reasons. An "Apply Plan" button executes the plan, streams real-time progress indicators, and reports the final execution outcome.

**Blocked by:** 03 (Destination Manifest Ledger and Change Detection), 04 (Non-Destructive Retention and Safe Archiving), 05 (Destination Conflict Detection and Preservation), 10 (Tauri Desktop Shell and Profile Management UI)

**Status:** resolved

- [x] "Preview" action in UI invokes `preview_profile(profile_id)` via Tauri IPC
- [x] Preview screen renders grouped summary badges and filterable lists of planned operations (`Create`, `Update`, `Archive`, `Conflict`, `Warn`)
- [x] Conflicting notes and warning notes are highlighted with plain-language explanations
- [x] "Apply Plan" button confirms changes and triggers `apply_plan` with live progress bars/counters
- [x] Completion report shows total files modified, duration, and any errors encountered
- [x] Re-triggering preview after apply shows clean state with all matching items in `Skip`
