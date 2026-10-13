# 07: Full Reconciliation and Recovery Engine

**What to build:**
A full reconciliation command (`vault-publisher reconcile`) and core method that recalculates the desired projection from scratch, compares it against the destination filesystem and manifest ledger, heals divergence caused by dropped filesystem events or offline edits, verifies destination integrity, safely archives unmatching items, and updates the manifest. Running reconciliation repeatedly on an unchanged source and destination is completely idempotent.

**Blocked by:** 04 (Non-Destructive Retention and Safe Archiving), 05 (Destination Conflict Detection and Preservation), 06 (Attachment Policies and Wikilink Resolution)

**Status:** ready-for-agent

- [ ] Implements `reconcile` method in `publisher-core` and exposes it via `vault-publisher reconcile <config> --profile <id>`
- [ ] Recalculates expected projection from scratch and compares against destination state and manifest records
- [ ] Automatically detects and reconciles orphaned files previously published under this profile
- [ ] Flags external modifications as conflicts without overwriting
- [ ] Idempotency test: executing `reconcile` twice consecutively on unchanged vaults yields 0 operations on the second pass
- [ ] Recovers cleanly if the manifest is temporarily missing or out-of-sync
