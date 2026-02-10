#[cfg(feature = "command")]
pub use makepad_shell_core::command::*;
pub use makepad_shell_core::error::ShellError;
#[cfg(feature = "menu-model")]
pub use makepad_shell_core::menu::*;
#[cfg(feature = "notification")]
pub use makepad_shell_core::notification::*;
#[cfg(feature = "shortcut")]
pub use makepad_shell_core::shortcut::*;
#[cfg(feature = "tray")]
pub use makepad_shell_core::tray::*;

#[cfg(feature = "context-menu")]
mod context_menu;
#[cfg(feature = "context-menu")]
pub use context_menu::{ContextMenu, popup_context_menu};

#[cfg(feature = "app-menu")]
mod app_menu;
#[cfg(feature = "app-menu")]
pub use app_menu::{AppMenu, clear_app_menu, set_app_menu};

#[cfg(feature = "tray")]
mod tray;
#[cfg(feature = "tray")]
pub use tray::{Tray, TrayHandle};

#[cfg(feature = "notification")]
mod notifications;
#[cfg(feature = "notification")]
pub use notifications::Notifications;
