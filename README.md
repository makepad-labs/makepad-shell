# makepad-shell

System shell UI primitives for Makepad, providing native menus, tray/status items, and notifications with unified data models.

**Features**
- App menu bar (App Menu) and menu models
- Native context menus
- System tray/status item icon and menu
- System notifications
- Shortcut and role models (About/Preferences/Quit, etc.)

**Platform Support**
| Feature | macOS | Windows | Linux |
| --- | --- | --- | --- |
| App menu bar | Implemented | Stub (Unsupported) | Stub (Unsupported) |
| Context menu | Implemented | Not implemented | Not implemented |
| Tray/status item | Implemented | Stub (Unsupported) | Stub (Unsupported) |
| Notifications | Implemented | Not implemented | Not implemented |
| Data models (makepad-shell-core) | Available | Available | Available |

**Install**
1. Install a Rust toolchain that supports the 2024 edition (latest stable recommended).
2. Clone the repo:

```bash
git clone https://github.com/makepad-labs/makepad-shell.git
cd makepad-shell
```

3. Build:

```bash
cargo build -p makepad-shell
```

To build only the core models:

```bash
cargo build -p makepad-shell-core
```

**Use In Your Project**
`makepad-shell` is not published on crates.io. Use a `path` or `git` dependency.

```toml
[dependencies]
makepad-shell = { path = "/path/to/makepad-shell/crates/shell" }
# or
makepad-shell-core = { path = "/path/to/makepad-shell/crates/core" }
```

Git dependency:

```toml
[dependencies]
makepad-shell = { git = "https://github.com/makepad-labs/makepad-shell", branch = "main" }
```

**Configuration**
1. Default features are empty. Enable only what you need.
2. Feature overview:
- `command`: base command type (`CommandId`)
- `shortcut`: shortcut types (`Shortcut`/`Modifiers`/`Key`)
- `menu-model`: menu models (depends on `command` + `shortcut`)
- `app-menu`: app menu API (depends on `menu-model`)
- `context-menu`: context menu API (depends on `menu-model`)
- `tray`: tray API + `TrayMenuModel` (depends on `command` + `shortcut`)
- `notification`: notification API + `Notification` (depends on `command`)
- `platforms`: platform backends (currently macOS)
3. Example: tray only (no notifications):

```toml
[dependencies]
makepad-shell = { git = "https://github.com/makepad-labs/makepad-shell.git", default-features = false, features = ["tray", "platforms"] }
```

App menu only:

```toml
[dependencies]
makepad-shell = { git = "https://github.com/makepad-labs/makepad-shell.git", default-features = false, features = ["app-menu", "platforms"] }
```

Context menu only:

```toml
[dependencies]
makepad-shell = { git = "https://github.com/makepad-labs/makepad-shell.git", default-features = false, features = ["context-menu", "platforms"] }
```

Menu models only (no platform backends):

```toml
[dependencies]
makepad-shell = { path = "/path/to/makepad-shell/crates/shell", default-features = false, features = ["menu-model"] }
```

**Examples**
Examples depend on `makepad-widgets` (git dependency). First build will fetch it.

- Tray example:

```bash
cargo run -p makepad-shell-tray
```

- Context menu + App menu example:

```bash
cargo run -p makepad-shell-context-menu
```

- Notification example:

```bash
cargo run -p makepad-shell-notification
```

**Project Layout**
- `crates/core`: cross-platform data models (menu, tray, notification, command, shortcut)
- `crates/platforms`: platform backends (macOS-focused)
- `crates/shell`: public API surface (core + platforms)
- `examples`: demo apps

**License**
MIT OR Apache-2.0
