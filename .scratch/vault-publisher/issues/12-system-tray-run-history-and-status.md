# 12: System Tray, Run History, and Background Status

**What to build:**
A persistent system tray menu and status monitor for the desktop app that keeps running when the main window is closed. Provides tray controls to trigger "Run Now", "Reconcile Now", and "Pause/Resume" per profile. The UI includes an Activity & Run History log viewer with searchable execution summaries, and a one-click "Export Diagnostics" report bundle for troubleshooting.

**Blocked by:** 09 (Background Agent Daemon with Single-Instance Lock), 11 (Desktop Plan Preview and Interactive Apply)

**Status:** resolved

- [x] System tray icon displays publishing status (idle, publishing, error, paused)
- [x] Tray menu provides quick actions: "Open Vault Publisher", "Run Now", "Reconcile Now", "Pause All", "Quit"
- [x] Application can minimize or close to tray and continue background watching
- [x] Activity Log viewer in the UI displays historical runs with timestamps, durations, and counts of files modified/archived/conflicted
- [x] Detailed log entries can be expanded to inspect individual file operations
- [x] "Export Diagnostics" button generates a sanitized diagnostic report containing system info, profile configs, and recent log snippets
