use std::cell::Cell;
use std::rc::Rc;

use makepad_widgets::*;
use makepad_shell::{
    AppMenu, CommandId, CommandItem, ContextMenu, MenuAnchor, MenuBarModel, MenuItem,
    MenuItemRole, MenuModel, MenuTrigger, Submenu, TopMenu, TopMenuRole,
};

const CMD_COPY: u64 = 1;
const CMD_PASTE: u64 = 2;
const CMD_TOGGLE_GRID: u64 = 3;
const CMD_DISABLED: u64 = 4;
const CMD_ZOOM_TO_FIT: u64 = 5;
const CMD_ABOUT: u64 = 100;
const CMD_PREFERENCES: u64 = 101;
const CMD_QUIT: u64 = 102;

#[cfg(target_os = "macos")]
static APP_MENU_COMMAND: std::sync::Mutex<Option<CommandId>> = std::sync::Mutex::new(None);

live_design! {
    use link::theme::*;
    use link::widgets::*;

    App = {{App}} {
        ui: <Root> {
            main_window = <Window> {
                window: {title: "Makepad Shell Context Menu"}
                body = <View> {
                    width: Fill,
                    height: Fill,
                    flow: Down,
                    align: {x: 0.5, y: 0.5},
                    spacing: 12,
                    show_bg: true,
                    draw_bg: {
                        color: #2b2b2b
                    }

                    <Label> {
                        draw_text: {color: #fff}
                        text: "Right-click for native context menu. App menu is wired to commands."
                    }

                    status_label = <Label> {
                        draw_text: {color: #9}
                        text: "Last command: (none)"
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
    show_grid: bool,
    #[rust]
    last_command: Option<CommandId>,
}

impl LiveRegister for App {
    fn live_register(cx: &mut Cx) {
        crate::makepad_widgets::live_design(cx);
    }
}

impl App {
    fn install_app_menu(&mut self) {
        #[cfg(target_os = "macos")]
        {
            let menu_bar = build_menu_bar(self.show_grid);
            let result = AppMenu::set(menu_bar, |cmd| {
                log!("app menu invoked: {}", cmd.as_u64());
                eprintln!("app menu invoked: {}", cmd.as_u64());
                if let Ok(mut slot) = APP_MENU_COMMAND.lock() {
                    *slot = Some(cmd);
                }
            });
            if let Err(err) = result {
                log!("set_app_menu failed: {:?}", err);
                eprintln!("set_app_menu failed: {:?}", err);
            }
        }
    }

    fn drain_app_menu_commands(&mut self, cx: &mut Cx) {
        #[cfg(target_os = "macos")]
        {
            let cmd = APP_MENU_COMMAND.lock().ok().and_then(|mut slot| slot.take());
            if let Some(cmd) = cmd {
                self.apply_command(cx, cmd);
            }
        }
    }

    fn update_status_label(&mut self, cx: &mut Cx) {
        let text = match self.last_command {
            Some(cmd) => format!("Last command: {} (show_grid: {})", cmd.as_u64(), self.show_grid),
            None => "Last command: (none)".to_string(),
        };
        self.ui.label(ids!(status_label)).set_text(cx, &text);
    }

    fn open_context_menu(&mut self, cx: &mut Cx, e: &MouseDownEvent) {
        let menu = build_menu(self.show_grid);
        let selected = Rc::new(Cell::new(None));
        let selected_cb = selected.clone();

        #[cfg(target_os = "macos")]
        {
            let (ns_view, ns_event) = macos_context();
            let result = ContextMenu::popup_macos(
                menu,
                MenuAnchor::Window {
                    x: e.abs.x as f32,
                    y: e.abs.y as f32,
                },
                MenuTrigger::MouseRight,
                ns_view,
                ns_event,
                Box::new(move |cmd: CommandId| {
                    log!("context menu invoked: {}", cmd.as_u64());
                    eprintln!("context menu invoked: {}", cmd.as_u64());
                    selected_cb.set(Some(cmd));
                }),
            );
            if let Err(err) = result {
                log!("context menu error: {:?}", err);
                eprintln!("context menu error: {:?}", err);
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            let _ = menu;
            log!("context menu example is macOS-only");
        }

        if let Some(cmd) = selected.get() {
            self.apply_command(cx, cmd);
        }
    }

    fn apply_command(&mut self, cx: &mut Cx, cmd: CommandId) {
        self.last_command = Some(cmd);
        log!("menu command: {}", cmd.as_u64());
        match cmd.as_u64() {
            CMD_TOGGLE_GRID => {
                self.show_grid = !self.show_grid;
                self.install_app_menu();
            }
            CMD_QUIT => {
                cx.quit();
            }
            _ => {}
        }
        self.update_status_label(cx);
    }
}

impl MatchEvent for App {
    fn handle_startup(&mut self, _cx: &mut Cx) {
        self.install_app_menu();
    }
}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.match_event(cx, event);
        #[cfg(target_os = "macos")]
        {
            self.install_app_menu();
        }
        if let Event::MouseDown(e) = event {
            if e.button.is_secondary() {
                self.open_context_menu(cx, e);
            }
        }
        self.drain_app_menu_commands(cx);
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}

fn build_menu(show_grid: bool) -> MenuModel {
    let mut toggle_grid = CommandItem::new(CommandId::new(CMD_TOGGLE_GRID).unwrap(), "Show Grid");
    toggle_grid.checked = show_grid;

    let mut disabled = CommandItem::new(CommandId::new(CMD_DISABLED).unwrap(), "Disabled Item");
    disabled.enabled = false;

    MenuModel::new(vec![
        MenuItem::Command(CommandItem::new(CommandId::new(CMD_COPY).unwrap(), "Copy")),
        MenuItem::Command(CommandItem::new(CommandId::new(CMD_PASTE).unwrap(), "Paste")),
        MenuItem::Command(CommandItem::new(CommandId::new(CMD_PASTE).unwrap(), "This is from makepad-shell-core")),
        MenuItem::Separator,
        MenuItem::Command(disabled),
        MenuItem::Submenu(Submenu::new(
            "View",
            vec![
                MenuItem::Command(toggle_grid),
                MenuItem::Command(CommandItem::new(
                    CommandId::new(CMD_ZOOM_TO_FIT).unwrap(),
                    "Zoom to Fit",
                )),
            ],
        )),
    ])
}

fn build_menu_bar(show_grid: bool) -> MenuBarModel {
    let about = CommandItem::new(CommandId::new(CMD_ABOUT).unwrap(), "About Makepad Shell")
        .with_role(MenuItemRole::About);
    let prefs = CommandItem::new(CommandId::new(CMD_PREFERENCES).unwrap(), "Preferences…")
        .with_role(MenuItemRole::Preferences);
    let quit = CommandItem::new(CommandId::new(CMD_QUIT).unwrap(), "Quit Makepad Shell")
        .with_role(MenuItemRole::Quit);

    let mut grid_toggle =
        CommandItem::new(CommandId::new(CMD_TOGGLE_GRID).unwrap(), "Show Grid");
    grid_toggle.checked = show_grid;

    let app_menu = TopMenu::new(
        "Makepad Shell",
        vec![
            MenuItem::Command(about),
            MenuItem::Separator,
            MenuItem::Command(prefs),
            MenuItem::Separator,
            MenuItem::Command(quit),
        ],
    )
    .with_role(TopMenuRole::App);

    let file_menu = TopMenu::new(
        "File",
        vec![
            MenuItem::Command(CommandItem::new(CommandId::new(CMD_COPY).unwrap(), "Copy")),
            MenuItem::Command(CommandItem::new(CommandId::new(CMD_PASTE).unwrap(), "Paste")),
        ],
    )
    .with_role(TopMenuRole::File);

    let view_menu = TopMenu::new(
        "View",
        vec![
            MenuItem::Command(grid_toggle),
            MenuItem::Command(CommandItem::new(
                CommandId::new(CMD_ZOOM_TO_FIT).unwrap(),
                "Zoom to Fit",
            )),
        ],
    )
    .with_role(TopMenuRole::View);

    MenuBarModel::new(vec![app_menu, file_menu, view_menu])
}

#[cfg(target_os = "macos")]
fn macos_context() -> (*mut core::ffi::c_void, *mut core::ffi::c_void) {
    use makepad_widgets::makepad_platform::os::apple::apple_sys::{class, msg_send, nil, sel, sel_impl, ObjcId};
    use makepad_widgets::makepad_platform::os::apple::macos::macos_app::with_macos_app;

    let view: ObjcId = with_macos_app(|app| {
        app.cocoa_windows
            .first()
            .map(|(_, view)| *view)
            .unwrap_or(nil)
    });

    let ns_app: ObjcId = unsafe { msg_send![class!(NSApplication), sharedApplication] };
    let event: ObjcId = unsafe { msg_send![ns_app, currentEvent] };

    let view_ptr = if view == nil {
        core::ptr::null_mut()
    } else {
        view as *mut core::ffi::c_void
    };

    let event_ptr = if event == nil {
        core::ptr::null_mut()
    } else {
        event as *mut core::ffi::c_void
    };

    (view_ptr, event_ptr)
}
