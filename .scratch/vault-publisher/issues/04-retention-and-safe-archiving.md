# 04: Non-Destructive Retention and Safe Archiving

**What to build:**
Safe retention handling for notes that were previously published and recorded in the manifest ledger, but either had their metadata changed to no longer match rules or were deleted from the source vault. Instead of deleting these files from the destination, the engine moves them to a configured archive directory (e.g. `.vault-publisher-archive/<timestamp>/`). Destination files not present in the manifest ledger are strictly ignored and never removed.

**Blocked by:** 03 (Destination Manifest Ledger and Change Detection)

**Status:** resolved

- [x] Planner detects when a manifest-managed destination file is no longer produced by source evaluation
- [x] Marked as `Archive` in the plan
- [x] During `apply`, unmatching files are moved to `.vault-publisher-archive/<timestamp>/` preserving relative path structure
- [x] Files present in the destination vault but absent from the manifest ledger are classified as unmanaged and untouched
- [x] Dry-run mode displays planned archives without touching the filesystem
- [x] Automated tests verify that unmatching files are archived safely, unmanaged files are left intact, and manifest is updated accordingly
