#![cfg(target_os = "macos")]

use core::ffi::c_void;
use std::cell::RefCell;

use makepad_shell_core::menu::*;
use objc2::encode::{Encode, Encoding, RefEncode};
use objc2::ffi::NSInteger;
use objc2::rc::Retained;
use objc2::runtime::{AnyObject, Bool, NSObject, Sel};
use objc2::{class, define_class, msg_send, sel, DefinedClass, MainThreadMarker, MainThreadOnly};

#[derive(Debug)]
pub enum MacMenuError {
    Unsupported,
    BadCommandId,
    NotOnMainThread,
}

pub fn popup_context_menu_macos(
    menu: MenuModel,
    anchor: MenuAnchor,
    _trigger: MenuTrigger,
    ns_view: *mut c_void,
    ns_event: *mut c_void,
    on_command: Box<dyn Fn(CommandId) + 'static>,
) -> Result<(), MacMenuError> {
    let mtm = main_thread_marker();

    let target = MenuTarget::new(on_command, mtm);
    let target_ptr = Retained::as_ptr(&target) as *mut AnyObject;
    let ctx = BuildContext {
        target: Some(target_ptr),
    };
    let menu = build_ns_menu_items(&menu.items, &ctx)?;

    unsafe { popup_menu(menu, anchor, ns_view, ns_event) }?;
    Ok(())
}

pub fn set_app_menu_macos(
    menu_bar: MenuBarModel,
    on_command: Box<dyn Fn(CommandId) + 'static>,
) -> Result<(), MacMenuError> {
    let mtm = main_thread_marker();

    let target = MenuTarget::new(on_command, mtm);
    let target_ptr = Retained::as_ptr(&target) as *mut AnyObject;
    APP_MENU_TARGET.with(|slot| {
        *slot.borrow_mut() = Some(target);
    });

    let ctx = BuildContext {
        target: Some(target_ptr),
    };
    let normalized = normalize_menu_bar(menu_bar);
    let has_app = has_app_menu(&normalized);

    if has_app {
        let current_menu = current_main_menu();
        let updated = APP_MENU_STATE.with(|slot| {
            if let Some(state) = slot.borrow_mut().as_mut() {
                if state.menu == current_menu {
                    if let Ok(true) =
                        try_update_main_menu(state.menu, &state.model, &normalized, &ctx)
                    {
                        state.model = normalized.clone();
                        return true;
                    }
                }
            }
            false
        });
        if updated {
            return Ok(());
        }
    }

    let main_menu = build_main_menu(&normalized, &ctx, has_app)?;

    unsafe {
        let ns_app: *mut AnyObject = msg_send![class!(NSApplication), sharedApplication];
        let _: () = msg_send![ns_app, setMainMenu: main_menu];
    }

    APP_MENU_STATE.with(|slot| {
        if has_app {
            *slot.borrow_mut() = Some(AppMenuState {
                model: normalized,
                menu: main_menu,
            });
        } else {
            *slot.borrow_mut() = None;
        }
    });

    Ok(())
}

pub fn build_ns_menu_from_model(menu: &MenuModel) -> Result<*mut AnyObject, MacMenuError> {
    let ctx = BuildContext { target: None };
    build_ns_menu_items(&menu.items, &ctx)
}

// ------------------------------
// Internal helpers
// ------------------------------

thread_local! {
    static APP_MENU_TARGET: RefCell<Option<Retained<MenuTarget>>> = RefCell::new(None);
}

struct AppMenuState {
    model: MenuBarModel,
    menu: *mut AnyObject,
}

thread_local! {
    static APP_MENU_STATE: RefCell<Option<AppMenuState>> = RefCell::new(None);
}

struct BuildContext {
    target: Option<*mut AnyObject>,
}

impl BuildContext {
    fn action(&self) -> Option<Sel> {
        if self.target.is_some() {
            Some(sel!(menuItemInvoked:))
        } else {
            None
        }
    }
}

fn build_ns_menu_items(items: &[MenuItem], ctx: &BuildContext) -> Result<*mut AnyObject, MacMenuError> {
    unsafe {
        // NSMenu *menu = [[NSMenu alloc] initWithTitle:@""];
        let menu: *mut AnyObject = msg_send![class!(NSMenu), alloc];
        let title = nsstring("");
        let menu: *mut AnyObject = msg_send![menu, initWithTitle: title];
        let _: () = msg_send![menu, setAutoenablesItems: false];

        for item in items {
            if let Some(mi) = build_ns_menu_item(item, ctx)? {
                let _: () = msg_send![menu, addItem: mi];
            }
        }

        Ok(menu)
    }
}

fn build_main_menu(
    menu_bar: &MenuBarModel,
    ctx: &BuildContext,
    has_app: bool,
) -> Result<*mut AnyObject, MacMenuError> {
    unsafe {
        let main_menu: *mut AnyObject = msg_send![class!(NSMenu), alloc];
        let title = nsstring("");
        let main_menu: *mut AnyObject = msg_send![main_menu, initWithTitle: title];
        let _: () = msg_send![main_menu, setAutoenablesItems: false];

        if !has_app {
            let default_app_menu = build_default_app_menu()?;
            let app_title = process_name_nsstring();
            let root = new_menu_item(app_title, None, nsstring(""));
            let _: () = msg_send![root, setSubmenu: default_app_menu];
            let _: () = msg_send![main_menu, addItem: root];
        }

        for menu in &menu_bar.menus {
            let title = if menu.role == Some(TopMenuRole::App) {
                process_name_nsstring()
            } else if menu.label.is_empty() {
                process_name_nsstring()
            } else {
                nsstring(&menu.label)
            };
            let root = new_menu_item(title, None, nsstring(""));
            let submenu = build_ns_menu_items(&menu.items, ctx)?;
            let _: () = msg_send![root, setSubmenu: submenu];
            let _: () = msg_send![main_menu, addItem: root];
        }

        Ok(main_menu)
    }
}

fn build_ns_menu_item(
    item: &MenuItem,
    ctx: &BuildContext,
) -> Result<Option<*mut AnyObject>, MacMenuError> {
    unsafe {
        match item {
            MenuItem::Separator => {
                // [NSMenuItem separatorItem]
                let sep: *mut AnyObject = msg_send![class!(NSMenuItem), separatorItem];
                Ok(Some(sep))
            }

            MenuItem::Command(cmd) => {
                // [[NSMenuItem alloc] initWithTitle:action:keyEquivalent:]
                let title = nsstring(&cmd.label);
                let (action, key_equiv) = if let Some(role) = cmd.role {
                    (role_selector(role), nsstring(role_key_equivalent(role)))
                } else {
                    (ctx.action(), nsstring(""))
                };
                let mi = new_menu_item(title, action, key_equiv);

                if let Some(target) = ctx.target {
                    let _: () = msg_send![mi, setTarget: target];
                }

                if cmd.role == Some(MenuItemRole::Services) {
                    let services_menu = build_services_menu()?;
                    let _: () = msg_send![mi, setSubmenu: services_menu];
                }

                let _: () = msg_send![mi, setEnabled: cmd.enabled];

                // checked -> state (0 off, 1 on)
                // NSControlStateValueOff = 0, On = 1
                let state: NSInteger = if cmd.checked { 1 } else { 0 };
                let _: () = msg_send![mi, setState: state];

                let tag = command_id_to_tag(cmd.id)?;
                let _: () = msg_send![mi, setTag: tag];

                Ok(Some(mi))
            }

            MenuItem::Submenu(sub) => {
                // Create submenu item:
                // NSMenuItem *root = [[NSMenuItem alloc] initWithTitle:action:keyEquivalent:]
                let title = nsstring(&sub.label);
                let root = new_menu_item(title, None, nsstring(""));

                let submenu = build_ns_menu_items(&sub.items, ctx)?;
                let _: () = msg_send![root, setSubmenu: submenu];

                Ok(Some(root))
            }
        }
    }
}

fn command_id_to_tag(id: CommandId) -> Result<NSInteger, MacMenuError> {
    let id_u64 = id.as_u64();
    if id_u64 > (isize::MAX as u64) {
        return Err(MacMenuError::BadCommandId);
    }
    Ok(id_u64 as NSInteger)
}

fn tag_to_command_id(tag: NSInteger) -> Option<CommandId> {
    if tag <= 0 {
        return None;
    }
    CommandId::new(tag as u64)
}

fn has_app_menu(model: &MenuBarModel) -> bool {
    model.menus.iter().any(|menu| menu.role == Some(TopMenuRole::App))
}

fn normalize_menu_bar(menu_bar: MenuBarModel) -> MenuBarModel {
    let mut indexed: Vec<(usize, TopMenu)> = menu_bar.menus.into_iter().enumerate().collect();
    indexed.sort_by_key(|(index, menu)| (top_menu_order(menu.role), *index));
    let mut menus: Vec<TopMenu> = indexed.into_iter().map(|(_, menu)| menu).collect();

    for menu in &mut menus {
        if menu.role == Some(TopMenuRole::App) {
            menu.items = normalize_app_menu_items(std::mem::take(&mut menu.items));
        }
    }

    MenuBarModel::new(menus)
}

fn top_menu_order(role: Option<TopMenuRole>) -> usize {
    match role {
        Some(TopMenuRole::App) => 0,
        Some(TopMenuRole::File) => 1,
        Some(TopMenuRole::Edit) => 2,
        Some(TopMenuRole::View) => 3,
        Some(TopMenuRole::Window) => 4,
        Some(TopMenuRole::Help) => 5,
        None => 100,
    }
}

fn normalize_app_menu_items(items: Vec<MenuItem>) -> Vec<MenuItem> {
    let mut custom = Vec::new();
    let mut roles: std::collections::HashMap<MenuItemRole, MenuItem> = std::collections::HashMap::new();

    for item in items {
        if let MenuItem::Command(cmd) = &item {
            if let Some(role) = cmd.role {
                if !roles.contains_key(&role) {
                    roles.insert(role, item);
                    continue;
                }
            }
        }
        custom.push(item);
    }

    let mut sections: Vec<Vec<MenuItem>> = Vec::new();
    if let Some(item) = roles.remove(&MenuItemRole::About) {
        sections.push(vec![item]);
    }
    if !custom.is_empty() {
        sections.push(custom);
    }
    if let Some(item) = roles.remove(&MenuItemRole::Preferences) {
        sections.push(vec![item]);
    }
    if let Some(item) = roles.remove(&MenuItemRole::Services) {
        sections.push(vec![item]);
    }

    let mut hide_group = Vec::new();
    if let Some(item) = roles.remove(&MenuItemRole::Hide) {
        hide_group.push(item);
    }
    if let Some(item) = roles.remove(&MenuItemRole::HideOthers) {
        hide_group.push(item);
    }
    if let Some(item) = roles.remove(&MenuItemRole::ShowAll) {
        hide_group.push(item);
    }
    if !hide_group.is_empty() {
        sections.push(hide_group);
    }

    if let Some(item) = roles.remove(&MenuItemRole::Quit) {
        sections.push(vec![item]);
    }

    for (_, item) in roles {
        sections.push(vec![item]);
    }

    let mut out = Vec::new();
    for section in sections {
        if !out.is_empty() {
            out.push(MenuItem::Separator);
        }
        out.extend(section);
    }
    out
}

fn menu_bar_shape_eq(old: &MenuBarModel, new: &MenuBarModel) -> bool {
    if old.menus.len() != new.menus.len() {
        return false;
    }
    for (old_menu, new_menu) in old.menus.iter().zip(new.menus.iter()) {
        if old_menu.role != new_menu.role {
            return false;
        }
        if !menu_items_shape_eq(&old_menu.items, &new_menu.items) {
            return false;
        }
    }
    true
}

fn menu_items_shape_eq(old_items: &[MenuItem], new_items: &[MenuItem]) -> bool {
    if old_items.len() != new_items.len() {
        return false;
    }
    for (old_item, new_item) in old_items.iter().zip(new_items.iter()) {
        if !menu_item_shape_eq(old_item, new_item) {
            return false;
        }
    }
    true
}

fn menu_item_shape_eq(old_item: &MenuItem, new_item: &MenuItem) -> bool {
    match (old_item, new_item) {
        (MenuItem::Separator, MenuItem::Separator) => true,
        (MenuItem::Command(old_cmd), MenuItem::Command(new_cmd)) => old_cmd.role == new_cmd.role,
        (MenuItem::Submenu(old_sub), MenuItem::Submenu(new_sub)) => {
            menu_items_shape_eq(&old_sub.items, &new_sub.items)
        }
        _ => false,
    }
}

fn try_update_main_menu(
    menu: *mut AnyObject,
    old: &MenuBarModel,
    new: &MenuBarModel,
    ctx: &BuildContext,
) -> Result<bool, MacMenuError> {
    if !menu_bar_shape_eq(old, new) {
        return Ok(false);
    }
    unsafe {
        for (index, (old_menu, new_menu)) in old.menus.iter().zip(new.menus.iter()).enumerate() {
            let item: *mut AnyObject = msg_send![menu, itemAtIndex: index as NSInteger];
            if item.is_null() {
                return Ok(false);
            }
            if new_menu.role == Some(TopMenuRole::App) || old_menu.label != new_menu.label {
                let title = if new_menu.role == Some(TopMenuRole::App) {
                    process_name_nsstring()
                } else if new_menu.label.is_empty() {
                    process_name_nsstring()
                } else {
                    nsstring(&new_menu.label)
                };
                let _: () = msg_send![item, setTitle: title];
            }
            let submenu: *mut AnyObject = msg_send![item, submenu];
            if submenu.is_null() {
                return Ok(false);
            }
            update_menu_items(submenu, &old_menu.items, &new_menu.items, ctx)?;
        }
    }
    Ok(true)
}

fn update_menu_items(
    menu: *mut AnyObject,
    old_items: &[MenuItem],
    new_items: &[MenuItem],
    ctx: &BuildContext,
) -> Result<(), MacMenuError> {
    unsafe {
        for (index, (old_item, new_item)) in old_items.iter().zip(new_items.iter()).enumerate() {
            let item: *mut AnyObject = msg_send![menu, itemAtIndex: index as NSInteger];
            if item.is_null() {
                return Ok(());
            }
            match (old_item, new_item) {
                (MenuItem::Separator, MenuItem::Separator) => {}
                (MenuItem::Command(old_cmd), MenuItem::Command(new_cmd)) => {
                    update_command_item(item, old_cmd, new_cmd, ctx)?;
                }
                (MenuItem::Submenu(old_sub), MenuItem::Submenu(new_sub)) => {
                    if old_sub.label != new_sub.label {
                        let title = nsstring(&new_sub.label);
                        let _: () = msg_send![item, setTitle: title];
                    }
                    let submenu: *mut AnyObject = msg_send![item, submenu];
                    if !submenu.is_null() {
                        update_menu_items(submenu, &old_sub.items, &new_sub.items, ctx)?;
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn update_command_item(
    item: *mut AnyObject,
    old_cmd: &CommandItem,
    new_cmd: &CommandItem,
    _ctx: &BuildContext,
) -> Result<(), MacMenuError> {
    unsafe {
        if old_cmd.label != new_cmd.label {
            let title = nsstring(&new_cmd.label);
            let _: () = msg_send![item, setTitle: title];
        }
        if old_cmd.enabled != new_cmd.enabled {
            let _: () = msg_send![item, setEnabled: new_cmd.enabled];
        }
        if old_cmd.checked != new_cmd.checked {
            let state: NSInteger = if new_cmd.checked { 1 } else { 0 };
            let _: () = msg_send![item, setState: state];
        }
        if old_cmd.id != new_cmd.id {
            let tag = command_id_to_tag(new_cmd.id)?;
            let _: () = msg_send![item, setTag: tag];
        }
        if new_cmd.role == Some(MenuItemRole::Services) {
            let _ = build_services_menu();
        }
    }
    Ok(())
}

fn main_thread_marker() -> MainThreadMarker {
    MainThreadMarker::new().unwrap_or_else(|| unsafe { MainThreadMarker::new_unchecked() })
}

fn current_main_menu() -> *mut AnyObject {
    unsafe {
        let ns_app: *mut AnyObject = msg_send![class!(NSApplication), sharedApplication];
        let menu: *mut AnyObject = msg_send![ns_app, mainMenu];
        menu
    }
}

fn build_default_app_menu() -> Result<*mut AnyObject, MacMenuError> {
    unsafe {
        let menu: *mut AnyObject = msg_send![class!(NSMenu), alloc];
        let menu: *mut AnyObject = msg_send![menu, initWithTitle: nsstring("")];
        let _: () = msg_send![menu, setAutoenablesItems: false];

        let app_name = process_name_nsstring();
        let about_title = string_by_appending("About ", app_name);
        let about_item = new_menu_item(about_title, Some(sel!(orderFrontStandardAboutPanel:)), nsstring(""));
        let _: () = msg_send![menu, addItem: about_item];

        let sep: *mut AnyObject = msg_send![class!(NSMenuItem), separatorItem];
        let _: () = msg_send![menu, addItem: sep];

        let quit_title = string_by_appending("Quit ", app_name);
        let quit_item = new_menu_item(quit_title, Some(sel!(terminate:)), nsstring("q"));
        let _: () = msg_send![menu, addItem: quit_item];

        Ok(menu)
    }
}

fn build_services_menu() -> Result<*mut AnyObject, MacMenuError> {
    unsafe {
        let menu: *mut AnyObject = msg_send![class!(NSMenu), alloc];
        let menu: *mut AnyObject = msg_send![menu, initWithTitle: nsstring("Services")];
        let _: () = msg_send![menu, setAutoenablesItems: false];
        let ns_app: *mut AnyObject = msg_send![class!(NSApplication), sharedApplication];
        let _: () = msg_send![ns_app, setServicesMenu: menu];
        Ok(menu)
    }
}

fn process_name_nsstring() -> *mut AnyObject {
    unsafe {
        let info: *mut AnyObject = msg_send![class!(NSProcessInfo), processInfo];
        let name: *mut AnyObject = msg_send![info, processName];
        name
    }
}

fn string_by_appending(prefix: &str, tail: *mut AnyObject) -> *mut AnyObject {
    unsafe {
        let head = nsstring(prefix);
        let combined: *mut AnyObject = msg_send![head, stringByAppendingString: tail];
        combined
    }
}

fn new_menu_item(title: *mut AnyObject, action: Option<Sel>, key_equiv: *mut AnyObject) -> *mut AnyObject {
    unsafe {
        let item: *mut AnyObject = msg_send![class!(NSMenuItem), alloc];
        let item: *mut AnyObject =
            msg_send![item, initWithTitle: title, action: action, keyEquivalent: key_equiv];
        item
    }
}

fn role_selector(role: MenuItemRole) -> Option<Sel> {
    match role {
        MenuItemRole::About => Some(sel!(orderFrontStandardAboutPanel:)),
        MenuItemRole::Preferences => Some(sel!(showPreferences:)),
        MenuItemRole::Services => None,
        MenuItemRole::Hide => Some(sel!(hide:)),
        MenuItemRole::HideOthers => Some(sel!(hideOtherApplications:)),
        MenuItemRole::ShowAll => Some(sel!(unhideAllApplications:)),
        MenuItemRole::Quit => Some(sel!(terminate:)),
        MenuItemRole::Minimize => Some(sel!(performMiniaturize:)),
        MenuItemRole::Zoom => Some(sel!(performZoom:)),
        MenuItemRole::BringAllToFront => Some(sel!(arrangeInFront:)),
    }
}

fn role_key_equivalent(role: MenuItemRole) -> &'static str {
    match role {
        MenuItemRole::About => "",
        MenuItemRole::Preferences => ",",
        MenuItemRole::Services => "",
        MenuItemRole::Hide => "h",
        MenuItemRole::HideOthers => "",
        MenuItemRole::ShowAll => "",
        MenuItemRole::Quit => "q",
        MenuItemRole::Minimize => "m",
        MenuItemRole::Zoom => "",
        MenuItemRole::BringAllToFront => "",
    }
}

unsafe fn popup_menu(
    menu: *mut AnyObject,
    anchor: MenuAnchor,
    ns_view: *mut c_void,
    ns_event: *mut c_void,
) -> Result<(), MacMenuError> {
    if !ns_event.is_null() {
        if ns_view.is_null() {
            return Err(MacMenuError::Unsupported);
        }
        let _: () = msg_send![
            class!(NSMenu),
            popUpContextMenu: menu,
            withEvent: ns_event as *mut AnyObject,
            forView: ns_view as *mut AnyObject
        ];
        return Ok(());
    }

    let (location, view_ptr) = match anchor {
        MenuAnchor::Screen { x, y } => (
            NSPoint::new(x, y),
            std::ptr::null_mut::<AnyObject>(),
        ),
        MenuAnchor::Window { x, y } => {
            if ns_view.is_null() {
                return Err(MacMenuError::Unsupported);
            }
            (
                NSPoint::new(x, y),
                ns_view as *mut AnyObject,
            )
        }
    };

    let _: Bool = msg_send![
        menu,
        popUpMenuPositioningItem: std::ptr::null::<AnyObject>(),
        atLocation: location,
        inView: view_ptr
    ];
    Ok(())
}

define_class!(
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    #[ivars = MenuTargetIvars]
    struct MenuTarget;

    impl MenuTarget {
        #[unsafe(method(menuItemInvoked:))]
        fn menu_item_invoked(&self, sender: &AnyObject) {
            self.invoke_from_sender(sender);
        }

        #[unsafe(method(terminate:))]
        fn terminate(&self, sender: &AnyObject) {
            self.invoke_from_sender(sender);
            unsafe {
                let ns_app: *mut AnyObject = msg_send![class!(NSApplication), sharedApplication];
                let _: () = msg_send![ns_app, terminate: sender];
            }
        }

        #[unsafe(method(orderFrontStandardAboutPanel:))]
        fn order_front_standard_about_panel(&self, sender: &AnyObject) {
            self.invoke_from_sender(sender);
            unsafe {
                let ns_app: *mut AnyObject = msg_send![class!(NSApplication), sharedApplication];
                let _: () = msg_send![ns_app, orderFrontStandardAboutPanel: sender];
            }
        }

        #[unsafe(method(showPreferences:))]
        fn show_preferences(&self, sender: &AnyObject) {
            self.invoke_from_sender(sender);
        }

        #[unsafe(method(hide:))]
        fn hide(&self, sender: &AnyObject) {
            self.invoke_from_sender(sender);
            unsafe {
                let ns_app: *mut AnyObject = msg_send![class!(NSApplication), sharedApplication];
                let _: () = msg_send![ns_app, hide: sender];
            }
        }

        #[unsafe(method(hideOtherApplications:))]
        fn hide_other_applications(&self, sender: &AnyObject) {
            self.invoke_from_sender(sender);
            unsafe {
                let ns_app: *mut AnyObject = msg_send![class!(NSApplication), sharedApplication];
                let _: () = msg_send![ns_app, hideOtherApplications: sender];
            }
        }

        #[unsafe(method(unhideAllApplications:))]
        fn unhide_all_applications(&self, sender: &AnyObject) {
            self.invoke_from_sender(sender);
            unsafe {
                let ns_app: *mut AnyObject = msg_send![class!(NSApplication), sharedApplication];
                let _: () = msg_send![ns_app, unhideAllApplications: sender];
            }
        }

        #[unsafe(method(performMiniaturize:))]
        fn perform_miniaturize(&self, sender: &AnyObject) {
            self.invoke_from_sender(sender);
            unsafe {
                let ns_app: *mut AnyObject = msg_send![class!(NSApplication), sharedApplication];
                let window: *mut AnyObject = msg_send![ns_app, keyWindow];
                if !window.is_null() {
                    let _: () = msg_send![window, performMiniaturize: sender];
                }
            }
        }

        #[unsafe(method(performZoom:))]
        fn perform_zoom(&self, sender: &AnyObject) {
            self.invoke_from_sender(sender);
            unsafe {
                let ns_app: *mut AnyObject = msg_send![class!(NSApplication), sharedApplication];
                let window: *mut AnyObject = msg_send![ns_app, keyWindow];
                if !window.is_null() {
                    let _: () = msg_send![window, performZoom: sender];
                }
            }
        }

        #[unsafe(method(arrangeInFront:))]
        fn arrange_in_front(&self, sender: &AnyObject) {
            self.invoke_from_sender(sender);
            unsafe {
                let ns_app: *mut AnyObject = msg_send![class!(NSApplication), sharedApplication];
                let _: () = msg_send![ns_app, arrangeInFront: sender];
            }
        }
    }
);

struct MenuTargetIvars {
    on_command: Box<dyn Fn(CommandId) + 'static>,
}

impl MenuTarget {
    fn new(on_command: Box<dyn Fn(CommandId) + 'static>, mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(MenuTargetIvars { on_command });
        unsafe { msg_send![super(this), init] }
    }

    fn invoke_from_sender(&self, sender: &AnyObject) {
        let tag: NSInteger = unsafe { msg_send![sender, tag] };
        let Some(cmd) = tag_to_command_id(tag) else {
            return;
        };
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            (self.ivars().on_command)(cmd);
        }));
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct NSPoint {
    x: CGFloat,
    y: CGFloat,
}

impl NSPoint {
    fn new(x: f32, y: f32) -> Self {
        Self {
            x: x as CGFloat,
            y: y as CGFloat,
        }
    }
}

#[cfg(any(
    not(target_vendor = "apple"),
    all(target_os = "macos", target_pointer_width = "32")
))]
const NSPOINT_NAME: &str = "_NSPoint";

#[cfg(not(any(
    not(target_vendor = "apple"),
    all(target_os = "macos", target_pointer_width = "32")
)))]
const NSPOINT_NAME: &str = "CGPoint";

unsafe impl Encode for NSPoint {
    const ENCODING: Encoding = Encoding::Struct(
        NSPOINT_NAME,
        &[<CGFloat as Encode>::ENCODING, <CGFloat as Encode>::ENCODING],
    );
}

unsafe impl RefEncode for NSPoint {
    const ENCODING_REF: Encoding = Encoding::Pointer(&Self::ENCODING);
}

/// Build an autoreleased NSString* from Rust &str.
///
/// We use NSString::stringWithUTF8String: to avoid pulling more wrappers.
/// This is safe as long as the C string is valid UTF-8 and null-terminated.
fn nsstring(s: &str) -> *mut AnyObject {
    use std::ffi::CString;
    let c = CString::new(s).unwrap_or_else(|_| CString::new("").unwrap());
    unsafe {
        let ns: *mut AnyObject = msg_send![class!(NSString), stringWithUTF8String: c.as_ptr()];
        ns
    }
}

#[cfg(target_pointer_width = "64")]
type CGFloat = f64;

#[cfg(not(target_pointer_width = "64"))]
type CGFloat = f32;
