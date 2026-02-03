use makepad_shell_core::command::CommandId;
use makepad_shell_core::tray::TrayModel;

#[derive(Debug)]
pub enum WindowsTrayError {
    Unsupported,
}

pub struct WindowsTrayHandle;

pub fn create_tray_windows(
    _model: TrayModel,
    _on_command: Box<dyn Fn(CommandId) + 'static>,
    _on_activate: Box<dyn Fn() + 'static>,
) -> Result<WindowsTrayHandle, WindowsTrayError> {
    Err(WindowsTrayError::Unsupported)
}
