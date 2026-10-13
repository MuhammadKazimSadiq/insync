# 05: Destination Conflict Detection and Preservation

**What to build:**
Conflict detection that guards against overwriting edits made by external collaborators in the shared destination vault. Before applying an `Update` to a destination file, the engine checks whether the current destination content hash matches the recorded manifest hash. If they differ, the destination file was edited externally; the engine marks the action as `Conflict`, halts overwriting that specific file, logs an actionable warning, and proceeds safely with remaining non-conflicting files.

**Blocked by:** 03 (Destination Manifest Ledger and Change Detection)

**Status:** ready-for-agent

- [ ] Planner checks current destination content hash against the hash recorded in `.vault-publisher/manifest.json`
- [ ] If destination hash differs from recorded manifest hash, note action is set to `Conflict`
- [ ] Conflicting destination files are never overwritten or archived during `apply`
- [ ] Conflict events are clearly displayed in `preview` and logged in execution reports
- [ ] Non-conflicting files continue to be processed and applied normally in the same run
- [ ] Automated tests simulate external destination file edits and assert that conflicts are reported and file contents are preserved
