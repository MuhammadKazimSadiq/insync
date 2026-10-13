# Domain Docs

How engineering skills consume this repository's domain documentation.

## Before exploring, read these

- **`CONTEXT.md`** at the repo root: domain glossary and canonical vocabulary.
- **`docs/adr/`**: architectural decision records.

## Structure

Single-context repository layout:

```
/
├── CONTEXT.md
├── docs/
│   ├── adr/
│   │   ├── 0001-rule-based-one-way-publisher.md
│   │   ├── 0002-decoupled-rust-core-with-tauri-adapter.md
│   │   └── 0003-manifest-and-archive-safety.md
│   └── agents/
└── .scratch/
```

## Use the glossary's vocabulary

When naming domain concepts in ticket titles, PRs, comments, or test suites, always use terms as defined in `CONTEXT.md`. Avoid synonyms that the glossary explicitly avoids.
