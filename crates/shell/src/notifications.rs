use makepad_shell_core::command::CommandId;
use makepad_shell_core::notification::Notification;

use crate::ShellError;

pub struct Notifications;

impl Notifications {
    pub fn show(
        notification: Notification,
        on_command: impl Fn(CommandId) + 'static,
    ) -> Result<(), ShellError> {
        #[cfg(target_os = "macos")]
        {
            #[cfg(feature = "platforms")]
            {
                return makepad_shell_platforms::notification::macos::show_notification_macos(
                    notification,
                    Box::new(on_command),
                )
                .map_err(|_| ShellError::Unsupported);
            }
            #[cfg(not(feature = "platforms"))]
            {
                let _ = notification;
                let _ = on_command;
                return Err(ShellError::Unsupported);
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            let _ = notification;
            let _ = on_command;
            Err(ShellError::Unsupported)
        }
    }
}
