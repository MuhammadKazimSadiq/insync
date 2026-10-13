# 01: Minimal Rule-Based Publisher Tracer Bullet (CLI + Core Engine)

**What to build:**
A minimal, end-to-end command-line publisher and core engine that takes a basic JSON configuration file specifying a source vault, destination vault, and folder scope, scans Markdown notes with YAML frontmatter `status: completed`, and copies matching notes to the destination vault preserving relative directory paths. Includes a deterministic integration test running against temporary directories.

**Blocked by:** None (can start immediately)

**Status:** resolved

- [x] Core crate `publisher-core` and CLI binary `vault-publisher` compile and run
- [x] Accepts a configuration file containing source vault path, destination vault path, and single scope path
- [x] Parses YAML frontmatter from source Markdown files
- [x] Copies notes with frontmatter `status: completed` to the destination vault, preserving relative folder paths
- [x] Skips notes without frontmatter or with other status values
- [x] Integration test creates temporary source and destination vaults, executes `vault-publisher preview` and `apply`, and asserts destination files match expectations
