# makepad-shell

Makepad 的系统 Shell UI 基础组件库，提供原生菜单、托盘/状态栏和通知等能力，并以统一的模型抽象供 Makepad 应用调用。

**功能**
- 应用菜单栏（App Menu）与菜单模型
- 原生上下文菜单（Context Menu）
- 系统托盘/状态栏图标与菜单（Tray/Status Item）
- 系统通知（Notification）
- 菜单快捷键、菜单角色（About/Preferences/Quit 等）模型

**平台支持**
| 功能 | macOS | Windows | Linux |
| --- | --- | --- | --- |
| 菜单栏 | 已实现 | 占位（Unsupported） | 占位（Unsupported） |
| 上下文菜单 | 已实现 | 未实现 | 未实现 |
| 托盘/状态栏 | 已实现 | 占位（Unsupported） | 占位（Unsupported） |
| 通知 | 已实现 | 未实现 | 未实现 |
| 数据模型（makepad-shell-core） | 可用 | 可用 | 可用 |

**安装**
1. 安装支持 Rust 2024 edition 的工具链（建议使用最新稳定版）。
2. 克隆仓库：

```bash
git clone https://github.com/Project-Robius-China/makepad-shell.git
cd makepad-shell
```

3. 编译：

```bash
cargo build -p makepad-shell
```

如果只需要数据模型，可编译：

```bash
cargo build -p makepad-shell-core
```

**在项目中使用**
`makepad-shell` 目前未发布到 crates.io，推荐使用 `path` 或 `git` 依赖：

```toml
[dependencies]
makepad-shell = { path = "/path/to/makepad-shell/crates/shell" }
# 或者
makepad-shell-core = { path = "/path/to/makepad-shell/crates/core" }
```

也可以使用 Git 依赖：

```toml
[dependencies]
makepad-shell = { git = "https://github.com/Project-Robius-China/makepad-shell.git", branch = "main" }
```

**配置**
1. 默认不启用任何 feature，需要按需开启。
2. 功能特性说明：  
`command`：基础命令类型（`CommandId`）  
`shortcut`：快捷键类型（`Shortcut`/`Modifiers`/`Key`）  
`menu-model`：菜单模型（依赖 `command` + `shortcut`）  
`app-menu`：应用菜单栏 API（依赖 `menu-model`）  
`context-menu`：上下文菜单 API（依赖 `menu-model`）  
`tray`：托盘 API + `TrayMenuModel`（依赖 `command` + `shortcut`）  
`notification`：通知 API + `Notification`（依赖 `command`）  
`platforms`：平台实现（当前主要为 macOS）
3. 只启用所需功能示例（只要托盘，不要通知）：

```toml
[dependencies]
makepad-shell = { git = "https://github.com/Project-Robius-China/makepad-shell.git", default-features = false, features = ["tray", "platforms"] }
```

仅启用 App Menu：

```toml
[dependencies]
makepad-shell = { git = "https://github.com/Project-Robius-China/makepad-shell.git", default-features = false, features = ["app-menu", "platforms"] }
```

仅启用 Context Menu：

```toml
[dependencies]
makepad-shell = { git = "https://github.com/Project-Robius-China/makepad-shell.git", default-features = false, features = ["context-menu", "platforms"] }
```

4. 如果只需要模型且不希望引入平台层，不要开启 `platforms` 即可，例如仅使用菜单模型：

```toml
[dependencies]
makepad-shell = { path = "/path/to/makepad-shell/crates/shell", default-features = false, features = ["menu-model"] }
```

5. 需要平台实现时，请确保目标平台为 macOS（当前实现集中在 macOS）。

**示例**
示例依赖 `makepad-widgets`（Git 依赖），首次编译会拉取依赖。

- 托盘示例：

```bash
cargo run -p makepad-shell-tray
```

- 上下文菜单 + App Menu 示例：

```bash
cargo run -p makepad-shell-context-menu
```

- 通知示例：

```bash
cargo run -p makepad-shell-notification
```

**目录结构**
- `crates/core`：跨平台数据模型（菜单、托盘、通知）
- `crates/platforms`：平台实现（当前以 macOS 为主）
- `crates/shell`：对外统一 API（封装 core + platforms）
- `examples`：示例应用

**许可协议**
MIT OR Apache-2.0
