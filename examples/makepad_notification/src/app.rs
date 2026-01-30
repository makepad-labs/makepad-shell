use makepad_widgets::*;
use makepad_shell::{Notification, NotificationButton, NotificationSound, Notifications, CommandId};

const CMD_NOTIFY: u64 = 1;
const CMD_ACTION: u64 = 2;

#[cfg(target_os = "macos")]
static NOTIFICATION_COMMAND: std::sync::Mutex<Option<CommandId>> = std::sync::Mutex::new(None);

live_design! {
    use link::theme::*;
    use link::widgets::*;

    App = {{App}} {
        ui: <Root> {
            main_window = <Window> {
                window: {title: "Makepad Shell Notification"}
                body = <View> {
                    width: Fill,
                    height: Fill,
                    flow: Down,
                    align: {x: 0.5, y: 0.5},
                    spacing: 12,
                    show_bg: true,
                    draw_bg: { color: #2b2b2b }

                    <Label> {
                        draw_text: {color: #fff}
                        text: "macOS notification demo"
                    }

                    notify_btn = <Button> { text: "Send notification" }
                    action_btn = <Button> { text: "Send with action button" }

                    status_label = <Label> {
                        draw_text: {color: #9}
                        text: "Last action: (none)"
                    }
                }
            }
        }
    }
}

app_main!(App);

#[derive(Live, LiveHook)]
pub struct App {
    #[live]
    ui: WidgetRef,
    #[rust]
    last_command: Option<CommandId>,
    #[rust]
    send_count: u64,
}

impl LiveRegister for App {
    fn live_register(cx: &mut Cx) {
        crate::makepad_widgets::live_design(cx);
    }
}

impl App {
    fn send_notification(&mut self, with_action: bool) {
        #[cfg(target_os = "macos")]
        {
            self.send_count += 1;
            let ident = format!("makepad-shell.notification.demo.{}", self.send_count);
            let default_cmd = CommandId::new(CMD_NOTIFY).unwrap();
            let mut notification = Notification::new("Makepad Shell")
                .with_body(format!("Click the notification (#{})", self.send_count))
                .with_subtitle(format!("Demo {}", self.send_count))
                .with_identifier(ident)
                .with_default_action(default_cmd)
                .with_sound(NotificationSound::Default);

            if with_action {
                let action_cmd = CommandId::new(CMD_ACTION).unwrap();
                notification = notification.with_action_button(
                    NotificationButton::new(action_cmd, "Action"),
                );
            }

            let result = Notifications::show(notification, |cmd| {
                log!("notification invoked: {}", cmd.as_u64());
                if let Ok(mut slot) = NOTIFICATION_COMMAND.lock() {
                    *slot = Some(cmd);
                }
            });

            if let Err(err) = result {
                log!("notification error: {:?}", err);
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            let _ = with_action;
            log!("notification demo is macOS-only");
        }
    }

    fn drain_notification_commands(&mut self, cx: &mut Cx) {
        #[cfg(target_os = "macos")]
        {
            let cmd = NOTIFICATION_COMMAND.lock().ok().and_then(|mut slot| slot.take());
            if let Some(cmd) = cmd {
                self.last_command = Some(cmd);
                let text = format!("Last action: {}", cmd.as_u64());
                self.ui.label(ids!(status_label)).set_text(cx, &text);
            }
        }
    }
}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        if let Event::Actions(actions) = event {
            if self.ui.button(ids!(notify_btn)).clicked(&actions) {
                self.send_notification(false);
            }
            if self.ui.button(ids!(action_btn)).clicked(&actions) {
                self.send_notification(true);
            }
        }

        self.drain_notification_commands(cx);
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}
