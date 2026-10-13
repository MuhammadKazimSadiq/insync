# 02: Declarative Rule Engine and Scope Exclusions

**What to build:**
A flexible rule engine supporting declarative operators (`equals`, `notEquals`, `exists`, `notExists`, `contains`, `in`) and recursive boolean groups (`all` AND, `any` OR) with strict type fidelity (booleans vs strings, numbers, dates, tags). Path exclusion rules and safety checks prevent scanning `.obsidian`, private folders, symlink cycles, or destination paths escaping the vault boundaries.

**Blocked by:** 01 (Minimal Rule-Based Publisher Tracer Bullet)

**Status:** resolved

- [x] Rule evaluator supports `equals`, `notEquals`, `exists`, `notExists`, `contains`, `in`
- [x] Nested `all` and `any` logical combinations evaluate accurately
- [x] Strict type checking ensures boolean `true` does not match string `"true"`
- [x] Scope exclusions filter out `.obsidian`, hidden files, and user-defined patterns (e.g., `_private`, `**/*.draft.md`)
- [x] Path safety checks reject configurations where source equals destination, destination is nested inside source, or paths contain `..` escaping root
- [x] Malformed YAML in an individual note produces a warning and skips the note without aborting the run
- [x] Comprehensive unit and integration tests verify all operators, nested groups, and path exclusion behaviors
