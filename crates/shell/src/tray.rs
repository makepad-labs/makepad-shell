use makepad_shell_core::command::CommandId;
use makepad_shell_core::tray::{TrayIcon, TrayMenuModel, TrayModel};

use crate::ShellError;

pub struct TrayHandle {
    #[cfg(all(target_os = "macos", feature = "platforms"))]
    inner: makepad_shell_platforms::tray::macos::MacTrayHandle,
    #[cfg(all(target_os = "windows", feature = "platforms"))]
    inner: makepad_shell_platforms::tray::windows::WindowsTrayHandle,
}

impl TrayHandle {
    pub fn update_menu(&mut self, menu: TrayMenuModel) -> Result<(), ShellError> {
        #[cfg(all(target_os = "macos", feature = "platforms"))]
        {
            return self
                .inner
                .update_menu(&menu)
                .map_err(|_| ShellError::Unsupported);
        }
        #[cfg(all(target_os = "windows", feature = "platforms"))]
        {
            return self
                .inner
                .update_menu(&menu)
                .map_err(|_| ShellError::Unsupported);
        }
        #[cfg(all(target_os = "macos", not(feature = "platforms")))]
        {
            let _ = menu;
            Err(ShellError::Unsupported)
        }
        #[cfg(all(target_os = "windows", not(feature = "platforms")))]
        {
            let _ = menu;
            Err(ShellError::Unsupported)
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            let _ = menu;
            Err(ShellError::Unsupported)
        }
    }

    pub fn update_icon(&mut self, icon: TrayIcon) -> Result<(), ShellError> {
        #[cfg(all(target_os = "macos", feature = "platforms"))]
        {
            return self
                .inner
                .update_icon(&icon)
                .map_err(|_| ShellError::Unsupported);
        }
        #[cfg(all(target_os = "windows", feature = "platforms"))]
        {
            return self
                .inner
                .update_icon(&icon)
                .map_err(|_| ShellError::Unsupported);
        }
        #[cfg(all(target_os = "macos", not(feature = "platforms")))]
        {
            let _ = icon;
            Err(ShellError::Unsupported)
        }
        #[cfg(all(target_os = "windows", not(feature = "platforms")))]
        {
            let _ = icon;
            Err(ShellError::Unsupported)
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            let _ = icon;
            Err(ShellError::Unsupported)
        }
    }

    pub fn update_tooltip(&mut self, tooltip: Option<String>) -> Result<(), ShellError> {
        #[cfg(all(target_os = "macos", feature = "platforms"))]
        {
            return self
                .inner
                .update_tooltip(tooltip.as_deref())
                .map_err(|_| ShellError::Unsupported);
        }
        #[cfg(all(target_os = "windows", feature = "platforms"))]
        {
            return self
                .inner
                .update_tooltip(tooltip.as_deref())
                .map_err(|_| ShellError::Unsupported);
        }
        #[cfg(all(target_os = "macos", not(feature = "platforms")))]
        {
            let _ = tooltip;
            Err(ShellError::Unsupported)
        }
        #[cfg(all(target_os = "windows", not(feature = "platforms")))]
        {
            let _ = tooltip;
            Err(ShellError::Unsupported)
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            let _ = tooltip;
            Err(ShellError::Unsupported)
        }
    }
}

pub struct Tray;

impl Tray {
    pub fn create(
        model: TrayModel,
        on_command: impl Fn(CommandId) + 'static,
        on_activate: impl Fn() + 'static,
    ) -> Result<TrayHandle, ShellError> {
        #[cfg(target_os = "macos")]
        {
            #[cfg(feature = "platforms")]
            {
                let inner = makepad_shell_platforms::tray::macos::create_tray_macos(
                    model,
                    Box::new(on_command),
                    Box::new(on_activate),
                )
                .map_err(|_| ShellError::Unsupported)?;
                return Ok(TrayHandle { inner });
            }
            #[cfg(not(feature = "platforms"))]
            {
                let _ = model;
                let _ = on_command;
                let _ = on_activate;
                return Err(ShellError::Unsupported);
            }
        }
        #[cfg(target_os = "windows")]
        {
            #[cfg(feature = "platforms")]
            {
                let inner = makepad_shell_platforms::tray::windows::create_tray_windows(
                    model,
                    Box::new(on_command),
                    Box::new(on_activate),
                )
                .map_err(|_| ShellError::Unsupported)?;
                return Ok(TrayHandle { inner });
            }
            #[cfg(not(feature = "platforms"))]
            {
                let _ = model;
                let _ = on_command;
                let _ = on_activate;
                return Err(ShellError::Unsupported);
            }
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            let _ = model;
            let _ = on_command;
            let _ = on_activate;
            Err(ShellError::Unsupported)
        }
    }
}
