# 10: Tauri Desktop Shell and Profile Management UI

**What to build:**
A cross-platform desktop application built with Tauri 2 and a Svelte + TypeScript frontend. Exposes Tauri IPC commands connecting the UI to the underlying `publisher-core` configuration module. Provides a clean desktop UI to view, create, edit, validate, duplicate, and delete publishing profiles with path pickers, plain-language rule displays, and configuration persistence to disk.

**Blocked by:** 01 (Minimal Rule-Based Publisher Tracer Bullet), 02 (Declarative Rule Engine and Scope Exclusions)

**Status:** resolved

- [x] Tauri 2 application shell compiles and runs on macOS and Windows
- [x] Profile list view shows configured profiles, status badges (active, paused, error), and last run summaries
- [x] Profile editor provides folder pickers for source and destination vaults with immediate path validation
- [x] Plain-language rule builder allows constructing `all`/`any` conditions without editing raw JSON
- [x] Configuration is validated against `publisher-core` schemas and saved safely to disk
- [x] Supports exporting and importing profile JSON configurations
