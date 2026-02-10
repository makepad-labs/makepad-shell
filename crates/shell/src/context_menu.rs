#[cfg(target_os = "macos")]
use core::ffi::c_void;

use makepad_shell_core::command::CommandId;
use makepad_shell_core::menu::{MenuAnchor, MenuModel, MenuTrigger};

use crate::ShellError;

pub struct ContextMenu;

impl ContextMenu {
    pub fn popup(
        menu: MenuModel,
        anchor: MenuAnchor,
        trigger: MenuTrigger,
        on_command: impl Fn(CommandId) + 'static,
    ) -> Result<(), ShellError> {
        popup_context_menu(menu, anchor, trigger, on_command)
    }

    #[cfg(target_os = "macos")]
    pub fn popup_macos(
        menu: MenuModel,
        anchor: MenuAnchor,
        trigger: MenuTrigger,
        ns_view: *mut c_void,
        ns_event: *mut c_void,
        on_command: impl Fn(CommandId) + 'static,
    ) -> Result<(), ShellError> {
        #[cfg(feature = "platforms")]
        {
            return makepad_shell_platforms::menu::macos::popup_context_menu_macos(
                menu,
                anchor,
                trigger,
                ns_view,
                ns_event,
                Box::new(on_command),
            )
            .map_err(|_| ShellError::Unsupported);
        }
        #[cfg(not(feature = "platforms"))]
        {
            let _ = menu;
            let _ = anchor;
            let _ = trigger;
            let _ = ns_view;
            let _ = ns_event;
            let _ = on_command;
            return Err(ShellError::Unsupported);
        }
    }
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
