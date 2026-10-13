# 06: Attachment Policies and Wikilink Resolution

**What to build:**
Configurable attachment publishing policies (`markdown-only`, `explicit`, and `referenced`). In `referenced` mode, the engine scans matching Markdown notes for wikilink embeds (`![[attachment.png]]`) and standard Markdown image/file links, resolves them against allowed source scopes, copies eligible attachments to corresponding destination paths, records them in the manifest ledger, and warns on missing attachments without aborting the run.

**Blocked by:** 03 (Destination Manifest Ledger and Change Detection)

**Status:** resolved

- [x] Profile configuration supports `attachments.mode`: `"markdown-only"`, `"explicit"`, `"referenced"`
- [x] Parser extracts wikilink attachments (`![[file.ext]]`) and standard Markdown links (`![](path/file.ext)`)
- [x] In `referenced` mode, resolved attachments within allowed source scopes are scheduled for publishing and copied atomically
- [x] Non-whitelisted attachments or attachments outside permitted scopes are blocked from leaking into the destination
- [x] Missing attachment files emit a `Warn` in the plan and execution log, allowing Markdown note publication to proceed
- [x] Attachment dependencies are tracked in `.vault-publisher/manifest.json`
- [x] Automated integration tests verify that referenced attachments are copied, non-scoped attachments are excluded, and missing attachments produce warnings
