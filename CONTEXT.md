# Vault Publisher

Vault Publisher creates a controlled, rule-filtered shared vault from a private source vault. It models publishing as a one-way projection rather than a two-way synchronization engine.

## Language

### Core Concepts

**Source Vault**:
The private root directory containing original notes and attachments authored by the user.
_Avoid_: Master vault, primary folder, input vault

**Destination Vault**:
The shared root directory containing published notes and attachments intended for external collaborators.
_Avoid_: Target folder, replica vault, shared folder, output directory

**Profile**:
A named configuration binding a source vault, destination vault, scopes, exclusions, metadata rules, attachment policy, retention settings, and schedule.
_Avoid_: Job, sync pair, publishing task

**Scope**:
A relative path within the source vault that a profile is explicitly permitted to scan.
_Avoid_: Include folder, watched root

**Rule**:
A declarative condition evaluated against YAML frontmatter properties to determine note eligibility.
_Avoid_: Filter, predicate, query

**Projection**:
The desired target state of the destination vault computed by evaluating scopes, rules, and attachment policies against the source vault.
_Avoid_: Mirror, replica, sync state

**Attachment**:
A non-Markdown file (image, PDF, audio, canvas) referenced within a note or explicitly included in scope.
_Avoid_: Asset, binary, media file

**Plan**:
A calculated, immutable set of discrete operations (`Create`, `Update`, `Archive`, `Skip`, `Warn`, `Conflict`) representing the transition from current destination state to the desired projection.
_Avoid_: Diff, changeset, action queue

**Apply**:
The atomic execution of a plan against the destination filesystem, followed by an update to the manifest.
_Avoid_: Commit, sync push, write-out

**Run**:
A single end-to-end execution of a profile consisting of configuration loading, scanning, planning, and applying.
_Avoid_: Cycle, iteration, sync pass

**Reconciliation**:
A comprehensive full scan that recalculates the desired projection from scratch, repairs state divergence, detects external destination changes, and catches missed filesystem events.
_Avoid_: Full sync, re-indexing, repair scan

**Manifest**:
A versioned JSON ledger (`.vault-publisher/manifest.json`) in the destination vault that records all files currently managed by the publisher, their content hashes, rules, dependencies, and timestamps.
_Avoid_: Database, state file, catalog
