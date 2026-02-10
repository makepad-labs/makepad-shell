use makepad_shell_core::command::CommandId;
use makepad_shell_core::menu::MenuBarModel;

use crate::ShellError;

pub struct AppMenu;

impl AppMenu {
    pub fn set(
        menu: MenuBarModel,
        on_command: impl Fn(CommandId) + 'static,
    ) -> Result<(), ShellError> {
        set_app_menu(menu, on_command)
    }

    pub fn clear() -> Result<(), ShellError> {
        clear_app_menu()
    }
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
