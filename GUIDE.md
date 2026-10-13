# Vault Publisher User & Developer Guide

Vault Publisher is a rule-based, one-way synchronization tool designed for Obsidian. It selectively publishes curated notes from a private source vault into a shared destination vault while preserving collaborator edits, enforcing strict safety guarantees, and maintaining an immutable ledger.

---

## Table of Contents
- [Quick Start](#quick-start)
- [Option 1: Native Desktop Application (GUI)](#option-1-native-desktop-application-gui)
- [Option 2: In-Browser Web Preview](#option-2-in-browser-web-preview)
- [Option 3: Command-Line Interface (CLI & Daemon)](#option-3-command-line-interface-cli--daemon)
- [Themes (Light & Dark Mode)](#themes-light--dark-mode)
- [Configuration Reference](#configuration-reference)
- [Safety Guarantees & Conflict Handling](#safety-guarantees--conflict-handling)
- [Troubleshooting & FAQs](#troubleshooting--faqs)

---

## Quick Start

Make sure Rust (`cargo`) is available in your shell:
```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

To launch the native desktop application right away:
```bash
./open-app.sh
```

---

## Option 1: Native Desktop Application (GUI)

The desktop application is built with **Tauri 2**, **Rust**, and **Svelte 5**. It provides a native macOS window, a system tray menu, interactive diff previews, profile editing, and run history.

### How to Launch

You can launch the desktop app using any of the following methods from the repository root:

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
To recompile the frontend assets and rebuild the native desktop binary:
```bash
./open-app.sh --build
```
Or manually:
```bash
npm --prefix ui run build
cargo build -p vault-publisher-desktop --release
```

### Desktop UI Features
- **Publishing Profiles**: Create, edit, and delete sync configurations with folder scopes and exclusion patterns.
- **Rule Builder**: Define frontmatter matching rules (`status equals "completed"`, `share equals true`, etc.).
- **Plan Preview Screen**: Simulate sync operations before writing files. Inspect color-coded badges:
  - `Create`: New note meeting publication criteria.
  - `Update`: Note modified in the source vault.
  - `Skip`: Note unchanged or not meeting criteria.
  - `Archive`: Published note that was deleted or no longer matches rules (moved to `.vault-publisher-archive/`).
  - `Conflict`: Destination note edited externally by a collaborator (preserved safely, never overwritten).
- **Execution & Reconciliation**: Click **"Apply Plan Now"** or **"Reconcile"** for safe, idempotent sync.
- **Activity Log**: View recent execution runs, operation counts, and execution durations in milliseconds.
- **System Tray**: Access Vault Publisher quickly from the macOS menu bar.

---

## Option 2: In-Browser Web Preview

If you want to preview the UI, test layouts, or adjust themes in your browser (Safari, Chrome, Arc, Brave) without running the native desktop shell:

```bash
npm run ui
```
Then navigate to:
```
http://localhost:5173
```

> **Note**: In a standard browser, Vault Publisher automatically activates browser sandbox mode with `localStorage` persistence and simulated plan execution, allowing you to create profiles, edit rules, and test previews without requiring local filesystem access.

---

## Option 3: Command-Line Interface (CLI & Daemon)

The CLI tool (`vault-publisher`) provides headless, scriptable access for automated workflows, terminal users, and continuous background sync.

### CLI Setup
Ensure Rust is on your `PATH`:
```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

### Command Reference

#### 1. Validate Configuration
Validates syntax, vault paths, and rule predicates:
```bash
cargo run -p vault-publisher -- validate path/to/config.json
```

#### 2. Plan Preview (Dry-Run)
Calculates and prints the action plan without writing to disk:
```bash
cargo run -p vault-publisher -- preview path/to/config.json --profile <profile-id>
```

#### 3. Apply Plan
Executes the calculated plan, writing new files, updating modified files, and archiving deleted ones:
```bash
cargo run -p vault-publisher -- apply path/to/config.json --profile <profile-id>
```

#### 4. Full Reconciliation
Performs an idempotent, complete reconciliation between source, destination, and ledger:
```bash
cargo run -p vault-publisher -- reconcile path/to/config.json --profile <profile-id>
```

#### 5. Background Daemon Mode
Runs a background daemon with a debounced filesystem watcher and periodic reconciliation timer:
```bash
cargo run -p vault-publisher -- daemon path/to/config.json --profile <profile-id> --interval 300
```
- `--interval <seconds>`: Periodic reconciliation timer interval (default: 300 seconds).
- Uses file-level single-instance locking (`.lock`) to prevent concurrent daemon collisions.

---

## Themes (Light & Dark Mode)

Vault Publisher features full support for both **Light Mode** and **Dark Mode**, crafted with high-contrast semantic color tokens.

### How to Switch Themes:
- In the top right header of the UI, click the **☀️ Light Mode / 🌙 Dark Mode** toggle button.
- Your theme choice is automatically saved in `localStorage` and will persist across restarts.
- If no manual preference is set, the application automatically adapts to your macOS system theme (`prefers-color-scheme`).

### Semantic Color Coding:
| Action / State | Light Mode | Dark Mode | Meaning |
| :--- | :--- | :--- | :--- |
| **Create** | Emerald green (`#dcfce7`) | Vibrant mint (`#064e3b` / `#6ee7b7`) | New note published |
| **Update** | Sky blue (`#e0f2fe`) | Cyan (`#082f49` / `#7dd3fc`) | Existing note updated |
| **Skip** | Subtle slate (`#f1f5f9`) | Dark slate (`#1e293b` / `#94a3b8`) | Note unchanged |
| **Archive** | Amber (`#fef3c7`) | Orange amber (`#451a03` / `#fcd34d`) | Note safely archived |
| **Conflict** | Crimson red (`#fee2e2`) | Ruby red (`#450a0a` / `#fca5a5`) | External collaborator edit preserved |

---

## Configuration Reference

Profile configurations can be managed via the desktop UI or written directly in JSON (`~/.config/vault-publisher/config.json`):

```json
{
  "version": 1,
  "profiles": [
    {
      "id": "shared-notes",
      "name": "Shared Notes",
      "sourceVault": "/Users/username/Documents/PrivateVault",
      "destinationVault": "/Users/username/Documents/SharedVault",
      "scopes": [
        "Courses",
        "Projects/Public"
      ],
      "excludes": [
        ".obsidian",
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

### Supported Rule Operators:
- `equals`: Property value exactly matches string, boolean, or number.
- `notEquals`: Property does not match the value.
- `contains`: String or array property contains the given substring or element.
- `exists`: Property key exists in note YAML frontmatter.
- `notExists`: Property key does not exist.
- `in`: Property value is contained within a target list.

### Attachment Modes:
- `referenced` (Default): Only copies images and attachments that are actively wikilinked (`![[...]]`) in published notes.
- `explicit`: Copies all attachments present in the scope folder.
- `markdown-only`: Excludes all non-markdown files.

---

## Safety Guarantees & Conflict Handling

1. **One-Way Direction**: Writes exclusively from Private Source to Shared Destination; never modifies your source vault.
2. **Collaborator Conflict Detection**: Compares destination file hashes against `.vault-publisher/manifest.json`. If a collaborator modified a file directly in the shared vault, Vault Publisher will **never overwrite it**. It flags a conflict and preserves the collaborator's version.
3. **Non-Destructive Archiving**: Unpublished or deleted notes are safely moved to `.vault-publisher-archive/` in the destination vault with timestamps, rather than being deleted permanently.
4. **Atomic Staging**: Files are written to temporary staging files first and swapped into place atomically to protect against interrupted runs.

---

## Troubleshooting & FAQs

### Q: Why did a blank window open when running `./open-app.sh`?
A blank window typically occurs if:
1. Asset paths in `index.html` were absolute (`/assets/`) rather than relative (`./assets/`). This has been configured in `vite.config.ts`.
2. Tauri IPC capabilities were missing. In Tauri 2, `src-tauri/capabilities/default.json` grants the required permissions to the main window.
3. The binary needs to be rebuilt with updated assets. Run `./open-app.sh --build` to recompile cleanly.

### Q: How do I inspect the Webview console in the Desktop app?
DevTools are enabled in the desktop binary. With the Vault Publisher window open:
- Right-click anywhere in the app and select **Inspect Element**, or
- Press `Cmd + Option + I` to open WebKit Developer Tools.

### Q: Where does the app save its configuration on macOS?
Configuration is stored in JSON format at:
```
~/.config/vault-publisher/config.json
```
And manifest metadata for change tracking is stored inside the destination vault at:
```
<destination-vault>/.vault-publisher/manifest.json
```
