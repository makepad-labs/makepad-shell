#![cfg(target_os = "linux")]

use makepad_shell_core::command::CommandId;
use makepad_shell_core::menu::MenuBarModel;

#[derive(Debug)]
pub enum LinuxMenuError {
    Unsupported,
}

pub fn set_app_menu_linux(
    _menu: MenuBarModel,
    _on_command: Box<dyn Fn(CommandId) + 'static>,
) -> Result<(), LinuxMenuError> {
    Err(LinuxMenuError::Unsupported)
}
