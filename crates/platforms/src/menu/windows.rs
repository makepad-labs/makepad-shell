#![cfg(target_os = "windows")]

use makepad_shell_core::command::CommandId;
use makepad_shell_core::menu::MenuBarModel;

#[derive(Debug)]
pub enum WinMenuError {
    Unsupported,
}

pub fn set_app_menu_windows(
    _menu: MenuBarModel,
    _on_command: Box<dyn Fn(CommandId) + 'static>,
) -> Result<(), WinMenuError> {
    Err(WinMenuError::Unsupported)
}
