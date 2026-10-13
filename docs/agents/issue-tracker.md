# Issue Tracker: Local Markdown

Issues and specs for this repository live as markdown files under `.scratch/`.

## Conventions

- Feature directory: `.scratch/vault-publisher/`
- Spec location: `.scratch/vault-publisher/spec.md`
- Implementation tickets: one file per ticket at `.scratch/vault-publisher/issues/<NN>-<slug>.md`, numbered sequentially from `01` in dependency order. Never combine tickets into a single file.
- Triage state is recorded as a `Status:` line near the top of each issue file (default: `ready-for-agent`).
- Comments and discussion append to the bottom of the file under a `## Comments` heading.

## When a skill says "publish to the issue tracker"

Create a new file under `.scratch/vault-publisher/` (or `.scratch/vault-publisher/issues/`).

## When a skill says "fetch the relevant ticket"

Read the file at the referenced path. Users and agents pass the relative path or issue number.
