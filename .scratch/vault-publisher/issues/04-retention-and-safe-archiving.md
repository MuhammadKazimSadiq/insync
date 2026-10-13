# 04: Non-Destructive Retention and Safe Archiving

**What to build:**
Safe retention handling for notes that were previously published and recorded in the manifest ledger, but either had their metadata changed to no longer match rules or were deleted from the source vault. Instead of deleting these files from the destination, the engine moves them to a configured archive directory (e.g. `.vault-publisher-archive/<timestamp>/`). Destination files not present in the manifest ledger are strictly ignored and never removed.

**Blocked by:** 03 (Destination Manifest Ledger and Change Detection)

**Status:** ready-for-agent

- [ ] Planner detects when a manifest-managed destination file is no longer produced by source evaluation
- [ ] Marked as `Archive` in the plan
- [ ] During `apply`, unmatching files are moved to `.vault-publisher-archive/<timestamp>/` preserving relative path structure
- [ ] Files present in the destination vault but absent from the manifest ledger are classified as unmanaged and untouched
- [ ] Dry-run mode displays planned archives without touching the filesystem
- [ ] Automated tests verify that unmatching files are archived safely, unmanaged files are left intact, and manifest is updated accordingly
