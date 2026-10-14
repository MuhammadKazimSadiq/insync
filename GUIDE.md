# InSync User & Developer Guide

**InSync** is a sleek, rule-based, one-way folder synchronization tool. It selectively syncs curated files from a **Source Folder** into a **Destination Folder** while preserving external collaborator edits, enforcing strict safety guarantees, and maintaining an immutable change ledger.

---

## Table of Contents
- [Quick Start](#quick-start)
- [Option 1: Native Desktop Application (GUI)](#option-1-native-desktop-application-gui)
- [Option 2: In-Browser Web Preview](#option-2-in-browser-web-preview)
- [Option 3: Command-Line Interface (CLI & Daemon)](#option-3-command-line-interface-cli--daemon)
- [Nord Theme (Light & Dark Mode)](#nord-theme-light--dark-mode)
- [Synced Folders & Rule Filtering](#synced-folders--rule-filtering)
- [Safety Guarantees & Conflict Handling](#safety-guarantees--conflict-handling)
- [Troubleshooting & FAQs](#troubleshooting--faqs)

---

## Quick Start

Make sure Rust (`cargo`) is available in your shell:
```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

To launch the native desktop application:
```bash
./open-app.sh
```

---

## Option 1: Native Desktop Application (GUI)

The desktop application is built with **Tauri 2**, **Rust**, **Svelte 5**, and **Tailwind CSS**, styled with the official **Nord Palette** and clean Lucide iconography (no emojis).

### How to Launch

Launch using any of the following methods from the repository root:

1. **Using the one-line launcher script (Recommended):**
   ```bash
   ./open-app.sh
   ```

2. **Using the compiled release binary directly:**
   ```bash
   ./target/release/vault-publisher-desktop
   ```

3. **Using npm:**
   ```bash
   npm run desktop
   ```

4. **Using Cargo:**
   ```bash
   cargo run -p vault-publisher-desktop --release
   ```

### To Rebuild After Making Changes
```bash
./open-app.sh --build
```
Or manually:
```bash
npm --prefix ui run build
cargo build -p vault-publisher-desktop --release
```

### Desktop UI Features
- **Synced Folders Management**: Configure one or more folder mappings with source, destination, scope subfolders, and exclusion rules.
- **Native Finder Pickers**: Click "Browse..." next to Source Folder or Destination Folder to choose folders via native macOS Finder dialogs.
- **Multi-Rule Filter Builder**: Add, edit, or remove multiple filter predicates with operators (`equals`, `notEquals`, `contains`, `exists`, `notExists`, `in`).
- **Plan Preview Screen**: Simulate sync operations before writing files. Inspect color-coded badges:
  - `Create`: New file meeting publication criteria.
  - `Update`: File modified in the source folder.
  - `Skip`: File unchanged or not matching criteria.
  - `Archive`: Synced file that was deleted or no longer matches rules (safely archived in destination `.vault-publisher-archive/`).
  - `Conflict`: Destination file edited externally (preserved safely, never overwritten).
- **Execution & Reconciliation**: Click **"Apply Sync Now"** or **"Reconcile"** for safe, idempotent one-way syncing.
- **Activity Log**: View recent execution runs, operation counts, and execution durations.
- **Export Config**: Native file dialog to export your sync configuration safely.
- **macOS System Tray**: Access InSync from the menu bar.

---

## Option 2: In-Browser Web Preview

If you want to preview the UI, test layouts, or adjust themes in your browser (Safari, Chrome, Arc, Brave):

```bash
npm run ui
```
Then navigate to:
```
http://localhost:5173
```

> **Note**: In a standard browser, InSync automatically runs in sandbox mode with `localStorage` persistence and simulated execution.

---

## Option 3: Command-Line Interface (CLI & Daemon)

The CLI tool provides headless, scriptable access for automated workflows and continuous background sync.

### Command Reference

#### 1. Validate Configuration
```bash
cargo run -p vault-publisher -- validate path/to/config.json
```

#### 2. Plan Preview (Dry-Run)
```bash
cargo run -p vault-publisher -- preview path/to/config.json --profile <folder-id>
```

#### 3. Apply Sync Plan
```bash
cargo run -p vault-publisher -- apply path/to/config.json --profile <folder-id>
```

#### 4. Full Reconciliation
```bash
cargo run -p vault-publisher -- reconcile path/to/config.json --profile <folder-id>
```

#### 5. Background Daemon Mode
Runs a background daemon with a debounced filesystem watcher and periodic reconciliation timer:
```bash
cargo run -p vault-publisher -- daemon path/to/config.json --profile <folder-id> --interval 300
```

---

## Nord Theme (Light & Dark Mode)

InSync features the authentic **Nord Theme** for both dark and light modes:

### How to Switch Themes:
- In the top right header, click the **Light / Dark** toggle button.
- Your theme choice persists across restarts via `localStorage`.
- Automatically respects your macOS system preference if no manual selection has been made.

### Nord Palette Reference:
| Category | Token | Hex Code | Usage |
| :--- | :--- | :--- | :--- |
| **Polar Night** | `nord0` | `#2E3440` | Base background (Dark mode) / Primary text (Light mode) |
| **Polar Night** | `nord1` | `#3B4252` | Card & panel background (Dark mode) |
| **Polar Night** | `nord2` | `#434C5E` | Input & subtle component background (Dark mode) |
| **Polar Night** | `nord3` | `#4C566A` | Borders & muted dividers |
| **Snow Storm** | `nord4` | `#D8DEE9` | Light border / secondary text |
| **Snow Storm** | `nord5` | `#E5E9F0` | Light card background (Light mode) |
| **Snow Storm** | `nord6` | `#ECEFF4` | Base background (Light mode) / Light text (Dark mode) |
| **Frost** | `nord8` | `#88C0D0` | Arctic Blue accent (Dark mode) |
| **Frost** | `nord10` | `#5E81AC` | Deep Nord Blue accent (Light mode) |
| **Aurora** | `nord14` | `#A3BE8C` | Create / active green badge |
| **Aurora** | `nord13` | `#EBCB8B` | Archive amber badge |
| **Aurora** | `nord11` | `#BF616A` | Conflict / danger red badge |

---

## Synced Folders & Rule Filtering

Configurations are stored in JSON (`~/.config/insync/config.json`):

```json
{
  "version": 1,
  "profiles": [
    {
      "id": "work-sync",
      "name": "Work Documents Sync",
      "sourceVault": "/Users/username/SourceFolder",
      "destinationVault": "/Users/username/DestinationFolder",
      "scopes": [
        "Documents",
        "Projects/Shared"
      ],
      "excludes": [
        ".git",
        "_private",
        "*.draft.md"
      ],
      "rules": {
        "all": [
          {
            "property": "status",
            "operator": "equals",
            "value": "completed"
          },
          {
            "property": "share",
            "operator": "equals",
            "value": true
          }
        ]
      },
      "attachments": {
        "mode": "referenced"
      }
    }
  ]
}
```

---

## Safety Guarantees & Conflict Handling

1. **One-Way Direction**: Syncs exclusively from Source to Destination; never alters your source files.
2. **Collaborator Conflict Detection**: Compares destination file hashes against `.vault-publisher/manifest.json`. If a collaborator modified a file directly in the shared folder, InSync will **never overwrite it**.
3. **Non-Destructive Archiving**: Deleted source files are safely archived to `.vault-publisher-archive/` in the destination folder rather than destroyed.
4. **Atomic Staging**: Files are written to temporary staging files first and swapped into place atomically.
