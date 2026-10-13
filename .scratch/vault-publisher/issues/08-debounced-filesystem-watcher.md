# 08: Debounced Event-Driven Filesystem Watcher

**What to build:**
An event-driven filesystem watcher for source scopes in `publisher-core`. Watches for file creation, modification, deletion, and rename events across configured profile scopes. Uses event debouncing with a configurable quiet period (e.g., 2–5 seconds) so that rapid keystrokes or multi-file operations collapse into a single publishing run, emitting structured events and execution summaries.

**Blocked by:** 07 (Full Reconciliation and Recovery Engine)

**Status:** resolved

- [x] File watcher listens for events across all configured source scopes in a profile
- [x] Events are debounced with a configurable quiet window (default 2 seconds) to avoid thrashing during typing
- [x] Multiple file edits within the quiet window are batched into a single publishing run
- [x] Filters out noise from temporary files, `.obsidian` internal state, and excluded patterns
- [x] Triggers a publishing run and reports output through structured callbacks/events
- [x] Automated tests verify that bursts of filesystem events trigger only one debounced run after the quiet period
