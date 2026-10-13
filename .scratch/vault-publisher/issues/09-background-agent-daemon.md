# 09: Background Agent Daemon with Single-Instance Lock

**What to build:**
A background daemon mode (`vault-publisher daemon`) that runs continuously in headless mode. Acquires an OS file lock per profile to ensure only one publisher agent evaluates a given profile at any time. Coordinates both the debounced event watcher (for instant reaction) and a periodic background reconciliation timer (e.g. every 15 minutes) to ensure complete data consistency across sleeps, crashes, and network disconnections.

**Blocked by:** 08 (Debounced Event-Driven Filesystem Watcher)

**Status:** ready-for-agent

- [ ] CLI supports `vault-publisher daemon <config> --profile <id>`
- [ ] Acquires a single-instance file lock per profile, exiting gracefully with an error if another daemon instance is already active
- [ ] Concurrently manages debounced filesystem watcher and background periodic reconciliation timer
- [ ] Recovers gracefully from unexpected temporary file-access interruptions (e.g., cloud drive indexing locks)
- [ ] Handles OS termination signals (SIGINT, SIGTERM) to flush in-flight operations and release file locks cleanly
- [ ] Integration tests verify single-instance locking and periodic timer triggering
