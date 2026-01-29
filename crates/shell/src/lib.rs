pub use makepad_shell_core::menu::*;

#[derive(Debug)]
pub enum ShellError {
    Unsupported,
}

pub fn popup_context_menu(
    _menu: MenuModel,
    _anchor: MenuAnchor,
    _trigger: MenuTrigger,
    _on_command: impl Fn(CommandId) + 'static,
) -> Result<(), ShellError> {
    // 未来这里会转发到 platforms
    Err(ShellError::Unsupported)
}

pub fn set_app_menu(
    menu: MenuBarModel,
    on_command: impl Fn(CommandId) + 'static,
) -> Result<(), ShellError> {
    #[cfg(target_os = "macos")]
    {
        #[cfg(feature = "platforms")]
        {
            return makepad_shell_platforms::menu::macos::set_app_menu_macos(
                menu,
                Box::new(on_command),
            )
            .map_err(|_| ShellError::Unsupported);
        }
        #[cfg(not(feature = "platforms"))]
        {
            let _ = menu;
            let _ = on_command;
            return Err(ShellError::Unsupported);
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = menu;
        let _ = on_command;
        Err(ShellError::Unsupported)
    }
}

pub fn clear_app_menu() -> Result<(), ShellError> {
    set_app_menu(MenuBarModel::new(Vec::new()), |_| {})
}
