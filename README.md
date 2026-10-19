# InSync

<p align="center">
  <strong>Controlled, rule-based one-way folder and note synchronization.</strong>
</p>

<p align="center">
  <a href="https://github.com/MuhammadKazimSadiq/insync/releases/tag/v1.0.0"><img src="https://img.shields.io/badge/release-v1.0.0-blue.svg" alt="Release v1.0.0" /></a>
  <a href="https://github.com/MuhammadKazimSadiq/insync/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/MuhammadKazimSadiq/insync/ci.yml?branch=main&label=CI" alt="CI Status" /></a>
  <img src="https://img.shields.io/badge/platform-macOS%20%7C%20Windows-5E81AC.svg" alt="Platform Support" />
  <img src="https://img.shields.io/badge/theme-Nord-88C0D0.svg" alt="Nord Theme" />
</p>

---

## Overview

**InSync** is a high-performance desktop application and command-line engine built with **Rust** and **Tauri 2**. It selectively projects files from a private **Source Folder** into a shared **Destination Folder** with strict safety guarantees, collaborator conflict preservation, and atomic file operations.

---

## Primary Use-Case: Selective Obsidian Vault Publishing

InSync was born out of a common problem faced by knowledge workers, researchers, and writers using **[Obsidian](https://obsidian.md)**:

> *You have a single, private Obsidian vault containing years of interconnected thoughts, journals, confidential work notes, and client documents. You want to share a curated subset of notes with a team, collaborator, or public repository—without leaking private notes, unreferenced attachments, or unpublished drafts.*

Standard cloud sync solutions (Dropbox, Google Drive, iCloud, Git) operate on an **all-or-nothing** basis: either the entire vault is synced, or you must maintain cumbersome, disjointed secondary vaults.

### How InSync Solves This for Obsidian

1. **Metadata & Frontmatter Rule Engine**:
   InSync evaluates YAML frontmatter properties inside your Markdown notes before syncing:
   - **Publish Flags**: Sync only notes where `publish: true` or `status: "ready"`.
   - **Tags & Categories**: Include notes tagged with `#public`, `shared`, or specific topic tags.
   - **Flexible Logic**: Combine rules with nested `ALL` (AND) or `ANY` (OR) groups, regex pattern matching, and exact equality.
2. **Referenced Attachment Discovery**:
   Obsidian notes frequently embed images, audio, canvases, and PDFs via wikilinks (`![[diagram.png]]`) or standard Markdown links (`![](assets/spec.pdf)`). 
   - Instead of copying your entire multi-gigabyte `attachments/` directory, InSync parses the AST of matched notes and copies **only the attachments directly referenced by published notes**.
   - Your private images and unreferenced assets remain strictly private in your source vault.
3. **Collaborator Protection (Never Overwrite Shared Edits)**:
   If a team member edits a note in the destination folder, InSync checks the destination hash against its internal ledger (`.vault-publisher/manifest.json`). It detects the external change and **refuses to overwrite** your collaborator's work, flagging a conflict instead.
4. **Non-Destructive Archiving**:
   If you change a note’s frontmatter to `publish: false` or delete it from your source vault, InSync does **not** permanently erase it from the destination. Instead, it safely archives the destination copy into `.vault-publisher-archive/` with a timestamp, preserving revision history.
5. **Strictly One-Way**:
   InSync never writes to, modifies, or deletes anything in your Source Vault. Your primary knowledge base is completely immutable to the publisher.

---

## General Use-Case: General-Purpose Folder Synchronization

While tailored for Obsidian and Markdown knowledge bases, InSync functions as a fast, reliable, **general-purpose one-way folder synchronizer**:

- **Zero-Rule Pass-Through**: When no metadata rules are configured, InSync automatically operates in general file sync mode—synchronizing all files, folders, and documents inside your chosen scopes.
- **Selective Scopes & Exclusions**: Target specific subfolders (e.g. `docs/`, `build/dist/`, `specs/`) while explicitly excluding hidden directories, temporary files, or dependencies (`node_modules/`, `.git/`, `.DS_Store`).
- **Cryptographic Hash Ledger**: Tracks every file's SHA-256 digest in a lightweight JSON manifest to perform incremental transfers without redundant copies.
- **Atomic Staging**: Files are staged into temporary buffers before being atomically swapped into place, eliminating corrupted or partially written files even during unexpected interruptions.

---

## Key Features

- **Native Desktop App (macOS & Windows)**: Built with Tauri 2 and Svelte 5 for a responsive footprint (<30 MB memory).
- **Automatic Background Sync**: Real-time debounced file-system watcher (2-second quiet period) automatically syncs additions, modifications, and deletions as you work.
- **System Tray & Menu Bar Support**: Close the main window and let InSync run silently in your macOS Menu Bar or Windows System Tray.
- **Manual Reconciliation**: Run an on-demand full scan at any time to verify integrity and repair any out-of-sync files.
- **Nord Design System**: Native Dark and Light themes designed around the authentic [Nord Palette](https://www.nordtheme.com).
- **Native File Pickers**: Integrated OS Finder / Windows Explorer folder pickers for selecting source and destination directories.
- **Auditable Sync History**: Inspect detailed logs of every run, including individual file statuses (`Created`, `Updated`, `Archived`, `Skipped`, `Conflict`).

---

## Download & Installation

Pre-built binaries for macOS and Windows are available on the [**GitHub Releases**](https://github.com/MuhammadKazimSadiq/insync/releases/tag/v1.0.0) page:

| Operating System | Package Format | Download Link |
| :--- | :--- | :--- |
| **macOS (Apple Silicon / ARM64)** | `.dmg` Installer | [InSync_1.0.0_aarch64.dmg](https://github.com/MuhammadKazimSadiq/insync/releases/download/v1.0.0/InSync_1.0.0_aarch64.dmg) |
| **macOS (Portable Archive)** | `.app.tar.gz` | [InSync_aarch64.app.tar.gz](https://github.com/MuhammadKazimSadiq/insync/releases/download/v1.0.0/InSync_aarch64.app.tar.gz) |
| **Windows (x64)** | `.msi` Windows Installer | [InSync_1.0.0_x64_en-US.msi](https://github.com/MuhammadKazimSadiq/insync/releases/download/v1.0.0/InSync_1.0.0_x64_en-US.msi) |
| **Windows (x64)** | Standalone `.exe` Setup | [InSync_1.0.0_x64-setup.exe](https://github.com/MuhammadKazimSadiq/insync/releases/download/v1.0.0/InSync_1.0.0_x64-setup.exe) |

---

## Configuration Examples

Configurations are stored in standard JSON at `~/.config/insync/config.json` (or `%APPDATA%\insync\config.json` on Windows). You can configure them via the graphical UI or by editing the file directly.

### Example 1: Obsidian Vault with Metadata Rules

```json
{
  "version": 1,
  "profiles": [
    {
      "id": "public-docs",
      "name": "Public Knowledge Base",
      "sourceVault": "/Users/alice/Notes/PersonalVault",
      "destinationVault": "/Users/alice/Shared/TeamWiki",
      "scopes": ["Articles", "Guides", "Reference"],
      "exclusions": [".obsidian/**", "**/*.draft.md"],
      "rules": {
        "operator": "all",
        "predicates": [
          {
            "property": "publish",
            "operator": "equals",
            "value": true
          },
          {
            "property": "status",
            "operator": "equals",
            "value": "final"
          }
        ]
      },
      "attachments": {
        "mode": "referenced"
      },
      "watch": {
        "enabled": true,
        "debounceSeconds": 2
      }
    }
  ]
}
```

### Example 2: General Folder Sync (No Rules)

```json
{
  "version": 1,
  "profiles": [
    {
      "id": "project-dist",
      "name": "Project Build Mirror",
      "sourceVault": "/Users/alice/Projects/MyApp/dist",
      "destinationVault": "/Volumes/BackupDrive/MyAppDist",
      "scopes": [],
      "exclusions": ["*.tmp", "**/.DS_Store"],
      "rules": null,
      "attachments": {
        "mode": "all"
      },
      "watch": {
        "enabled": true,
        "debounceSeconds": 2
      }
    }
  ]
}
```

---

## Building from Source

### Prerequisites

- **Rust**: `rustc` & `cargo` 1.80+ ([Install Rust](https://rustup.rs/))
- **Node.js**: v20 or v22+ ([Node.js](https://nodejs.org/))
- **npm** (bundled with Node)

### Build & Run Desktop App

```bash
# Clone the repository
git clone https://github.com/MuhammadKazimSadiq/insync.git
cd insync

# Install dependencies
npm ci
npm --prefix ui ci

# Launch desktop app in development mode
./open-app.sh

# Or run cargo test to verify all test suites
cargo test
```

### CLI & Daemon Usage

InSync can also be run headlessly in CI/CD pipelines or servers via the CLI crate:

```bash
# Run a one-time projection run
cargo run -p vault-publisher-cli -- run --profile public-docs

# Run reconciliation scan
cargo run -p vault-publisher-cli -- reconcile --profile public-docs

# Start the continuous background watcher daemon
cargo run -p vault-publisher-cli -- daemon --profile public-docs
```

---

## Project Architecture

```
sync/
├── crates/
│   ├── publisher-core/          # Core synchronization engine, rules evaluator, AST attachment parser, manifest ledger
│   └── vault-publisher-cli/     # Headless command-line binary & background daemon
├── src-tauri/                   # Tauri 2 desktop backend (Rust), system tray, native dialogs, watcher management
├── ui/                          # Svelte 5 + Tailwind CSS frontend interface
├── .github/workflows/           # GitHub Actions CI & cross-platform release pipelines (macOS .dmg & Windows .msi)
├── CONTEXT.md                   # Single-context domain glossary & architectural terminology
└── GUIDE.md                     # Comprehensive user and developer reference manual
```

---

## License

This project is licensed under the [MIT License](LICENSE).
