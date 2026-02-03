use makepad_shell_core::command::CommandId;
use makepad_shell_core::tray::TrayModel;

#[derive(Debug)]
pub enum LinuxTrayError {
    Unsupported,
}

pub struct LinuxTrayHandle;

pub fn create_tray_linux(
    _model: TrayModel,
    _on_command: Box<dyn Fn(CommandId) + 'static>,
    _on_activate: Box<dyn Fn() + 'static>,
) -> Result<LinuxTrayHandle, LinuxTrayError> {
    Err(LinuxTrayError::Unsupported)
}
