# 03: Destination Manifest Ledger and Change Detection

**What to build:**
A destination manifest ledger (`.vault-publisher/manifest.json`) that records managed files, their source content hashes (SHA-256), destination content hashes, publishing timestamps, and profile metadata. The planner compares source hashes against manifest records to calculate fine-grained plan actions (`Create`, `Update`, `Skip`) so that subsequent runs skip untouched files and only copy modified content using atomic writes.

**Blocked by:** 01 (Minimal Rule-Based Publisher Tracer Bullet), 02 (Declarative Rule Engine and Scope Exclusions)

**Status:** resolved

- [x] Destination manifest ledger `.vault-publisher/manifest.json` is created and updated after every apply
- [x] Content hashes (SHA-256) are calculated for source notes and recorded in the manifest
- [x] Planner compares source hash against manifest hash: unchanged notes are marked `Skip`
- [x] Changed notes are marked `Update` and written via atomic staging files (`.tmp.vault-publisher-*`) before atomic rename
- [x] CLI `preview` prints a structured plan showing file counts and per-file action breakdown (`Create`, `Update`, `Skip`)
- [x] Re-running `apply` on an unchanged source vault results in zero file mutations
