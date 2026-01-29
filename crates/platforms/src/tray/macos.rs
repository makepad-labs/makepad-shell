#![cfg(target_os = "macos")]

use core::ffi::c_void;
use std::cell::RefCell;

use makepad_shell_core::menu::CommandId;
use makepad_shell_core::tray::{TrayIcon, TrayModel};
use objc2::ffi::NSInteger;
use objc2::rc::Retained;
use objc2::runtime::{AnyObject, Bool, NSObject, Sel};
use objc2::{class, define_class, msg_send, sel, DefinedClass, MainThreadMarker, MainThreadOnly};

use crate::menu::macos::{build_ns_menu_with_target, tag_to_command_id, try_update_ns_menu, MacMenuError};

#[derive(Debug)]
pub enum MacTrayError {
    Unsupported,
    BadIcon,
    NotOnMainThread,
    Menu(MacMenuError),
}

impl From<MacMenuError> for MacTrayError {
    fn from(err: MacMenuError) -> Self {
        MacTrayError::Menu(err)
    }
}

thread_local! {
    static TRAY_APP_DELEGATE: RefCell<Option<Retained<TrayAppDelegate>>> = RefCell::new(None);
}

pub struct MacTrayHandle {
    status_item: *mut AnyObject,
    _target: Retained<TrayTarget>,
    _menu: *mut AnyObject,
    model: makepad_shell_core::menu::MenuModel,
}

impl MacTrayHandle {
    pub fn update_menu(&mut self, menu: &makepad_shell_core::menu::MenuModel) -> Result<(), MacTrayError> {
        let target_ptr = Retained::as_ptr(&self._target) as *mut AnyObject;
        if let Ok(true) = try_update_ns_menu(self._menu, &self.model, menu, target_ptr) {
            self.model = menu.clone();
            return Ok(());
        }
        let menu_ptr = build_ns_menu_with_target(&menu.items, target_ptr)?;
        self._menu = menu_ptr;
        self._target.set_handles(self.status_item, menu_ptr);
        self.model = menu.clone();
        Ok(())
    }

    pub fn update_icon(&mut self, icon: &TrayIcon) -> Result<(), MacTrayError> {
        unsafe {
            let button: *mut AnyObject = msg_send![self.status_item, button];
            if button.is_null() {
                return Err(MacTrayError::Unsupported);
            }
            let image = build_ns_image(icon)?;
            let _: () = msg_send![button, setImage: image];
        }
        Ok(())
    }

    pub fn update_tooltip(&mut self, tooltip: Option<&str>) -> Result<(), MacTrayError> {
        unsafe {
            let button: *mut AnyObject = msg_send![self.status_item, button];
            if button.is_null() {
                return Err(MacTrayError::Unsupported);
            }
            let tip = tooltip.map(nsstring).unwrap_or(std::ptr::null_mut());
            let _: () = msg_send![button, setToolTip: tip];
        }
        Ok(())
    }
}

impl Drop for MacTrayHandle {
    fn drop(&mut self) {
        if self.status_item.is_null() {
            return;
        }
        unsafe {
            let status_bar: *mut AnyObject = msg_send![class!(NSStatusBar), systemStatusBar];
            let _: () = msg_send![status_bar, removeStatusItem: self.status_item];
        }
    }
}

pub fn create_tray_macos(
    model: TrayModel,
    on_command: Box<dyn Fn(CommandId) + 'static>,
    on_activate: Box<dyn Fn() + 'static>,
) -> Result<MacTrayHandle, MacTrayError> {
    let _mtm = main_thread_marker();
    install_app_delegate(_mtm);

    unsafe {
        let status_bar: *mut AnyObject = msg_send![class!(NSStatusBar), systemStatusBar];
        let status_item: *mut AnyObject =
            msg_send![status_bar, statusItemWithLength: NS_VARIABLE_STATUS_ITEM_LENGTH];
        if status_item.is_null() {
            return Err(MacTrayError::Unsupported);
        }

    let target = TrayTarget::new(on_command, on_activate, _mtm);
    let target_ptr = Retained::as_ptr(&target) as *mut AnyObject;
    let menu = build_ns_menu_with_target(&model.menu.items, target_ptr)?;

        let button: *mut AnyObject = msg_send![status_item, button];
        if button.is_null() {
            return Err(MacTrayError::Unsupported);
        }

        let image = build_ns_image(&model.icon)?;
        let _: () = msg_send![button, setImage: image];

        if let Some(tooltip) = model.tooltip.as_ref() {
            let tooltip = nsstring(tooltip);
            let _: () = msg_send![button, setToolTip: tooltip];
        }

        let _: () = msg_send![button, setTarget: target_ptr];
        let _: () = msg_send![button, setAction: sel!(statusItemInvoked:)];
        let mask = NS_LEFT_MOUSE_DOWN_MASK | NS_RIGHT_MOUSE_DOWN_MASK;
        let _: u64 = msg_send![button, sendActionOn: mask];

        // Fill target ivars with handles now that we have them.
        target.set_handles(status_item, menu);

        Ok(MacTrayHandle {
            status_item,
            _target: target,
            _menu: menu,
            model: model.menu.clone(),
        })
    }
}

// ------------------------------
// Internal helpers
// ------------------------------

const NS_VARIABLE_STATUS_ITEM_LENGTH: CGFloat = -1.0;
const NS_LEFT_MOUSE_DOWN_MASK: u64 = 1 << 1;
const NS_RIGHT_MOUSE_DOWN_MASK: u64 = 1 << 3;
const NS_RIGHT_MOUSE_DOWN: NSInteger = 3;
const NS_RIGHT_MOUSE_UP: NSInteger = 4;
const NS_RIGHT_MOUSE_DRAGGED: NSInteger = 7;

fn main_thread_marker() -> MainThreadMarker {
    MainThreadMarker::new().unwrap_or_else(|| unsafe { MainThreadMarker::new_unchecked() })
}

fn install_app_delegate(mtm: MainThreadMarker) {
    TRAY_APP_DELEGATE.with(|slot| {
        if slot.borrow().is_some() {
            return;
        }
        unsafe {
            let ns_app: *mut AnyObject = msg_send![class!(NSApplication), sharedApplication];
            let fallback: *mut AnyObject = msg_send![ns_app, delegate];
            let delegate = TrayAppDelegate::new(fallback, mtm);
            let delegate_ptr = Retained::as_ptr(&delegate) as *mut AnyObject;
            let _: () = msg_send![ns_app, setDelegate: delegate_ptr];
            *slot.borrow_mut() = Some(delegate);
        }
    });
}

fn build_ns_image(icon: &TrayIcon) -> Result<*mut AnyObject, MacTrayError> {
    unsafe {
        match icon {
            TrayIcon::Png { bytes, is_template } => {
                if bytes.is_empty() {
                    return Err(MacTrayError::BadIcon);
                }
                let data: *mut AnyObject = msg_send![
                    class!(NSData),
                    dataWithBytes: bytes.as_ptr() as *const c_void,
                    length: bytes.len()
                ];
                if data.is_null() {
                    return Err(MacTrayError::BadIcon);
                }
                let image: *mut AnyObject = msg_send![class!(NSImage), alloc];
                let image: *mut AnyObject = msg_send![image, initWithData: data];
                if image.is_null() {
                    return Err(MacTrayError::BadIcon);
                }
                if *is_template {
                    let _: () = msg_send![image, setTemplate: true];
                }
                Ok(image)
            }
        }
    }
}

define_class!(
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    #[ivars = TrayTargetIvars]
    struct TrayTarget;

    impl TrayTarget {
        #[unsafe(method(statusItemInvoked:))]
        fn status_item_invoked(&self, _sender: &AnyObject) {
            self.handle_status_item();
        }

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

struct TrayTargetIvars {
    on_command: Box<dyn Fn(CommandId) + 'static>,
    on_activate: Box<dyn Fn() + 'static>,
    status_item: std::cell::Cell<*mut AnyObject>,
    menu: std::cell::Cell<*mut AnyObject>,
}

impl TrayTarget {
    fn new(
        on_command: Box<dyn Fn(CommandId) + 'static>,
        on_activate: Box<dyn Fn() + 'static>,
        mtm: MainThreadMarker,
    ) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(TrayTargetIvars {
            on_command,
            on_activate,
            status_item: std::cell::Cell::new(std::ptr::null_mut()),
            menu: std::cell::Cell::new(std::ptr::null_mut()),
        });
        unsafe { msg_send![super(this), init] }
    }

    fn set_handles(&self, status_item: *mut AnyObject, menu: *mut AnyObject) {
        self.ivars().status_item.set(status_item);
        self.ivars().menu.set(menu);
    }

    fn handle_status_item(&self) {
        unsafe {
            let ns_app: *mut AnyObject = msg_send![class!(NSApplication), sharedApplication];
            let event: *mut AnyObject = msg_send![ns_app, currentEvent];
            let mut is_right_click = false;
            if !event.is_null() {
                let event_type: NSInteger = msg_send![event, type];
                if event_type == NS_RIGHT_MOUSE_DOWN
                    || event_type == NS_RIGHT_MOUSE_UP
                    || event_type == NS_RIGHT_MOUSE_DRAGGED
                {
                    is_right_click = true;
                }
            }

            if is_right_click {
                let menu = self.ivars().menu.get();
                let status_item = self.ivars().status_item.get();
                if !menu.is_null() && !status_item.is_null() {
                    let _: () = msg_send![status_item, popUpStatusItemMenu: menu];
                }
            } else {
                activate_application();
                let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    (self.ivars().on_activate)();
                }));
            }
        }
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

define_class!(
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    #[ivars = TrayAppDelegateIvars]
    struct TrayAppDelegate;

    impl TrayAppDelegate {
        #[unsafe(method(applicationShouldHandleReopen:hasVisibleWindows:))]
        fn application_should_handle_reopen(
            &self,
            _app: &AnyObject,
            _has_visible: Bool,
        ) -> Bool {
            unsafe {
                activate_application();
            }
            Bool::YES
        }

        #[unsafe(method(respondsToSelector:))]
        fn responds_to_selector(&self, selector: Sel) -> Bool {
            if selector == sel!(applicationShouldHandleReopen:hasVisibleWindows:) {
                return Bool::YES;
            }
            let fallback = self.fallback_ptr();
            if fallback.is_null() {
                return Bool::NO;
            }
            unsafe { msg_send![fallback, respondsToSelector: selector] }
        }

        #[unsafe(method(forwardingTargetForSelector:))]
        fn forwarding_target_for_selector(&self, selector: Sel) -> *mut AnyObject {
            if selector == sel!(applicationShouldHandleReopen:hasVisibleWindows:) {
                return std::ptr::null_mut();
            }
            let fallback = self.fallback_ptr();
            if fallback.is_null() {
                return std::ptr::null_mut();
            }
            let responds: Bool = unsafe { msg_send![fallback, respondsToSelector: selector] };
            if responds.as_bool() {
                fallback
            } else {
                std::ptr::null_mut()
            }
        }
    }
);

struct TrayAppDelegateIvars {
    fallback: Option<Retained<AnyObject>>,
}

impl TrayAppDelegate {
    fn new(fallback: *mut AnyObject, mtm: MainThreadMarker) -> Retained<Self> {
        let fallback = unsafe { Retained::retain(fallback) };
        let this = Self::alloc(mtm).set_ivars(TrayAppDelegateIvars { fallback });
        unsafe { msg_send![super(this), init] }
    }

    fn fallback_ptr(&self) -> *mut AnyObject {
        self.ivars()
            .fallback
            .as_ref()
            .map(|f| Retained::as_ptr(f) as *mut AnyObject)
            .unwrap_or(std::ptr::null_mut())
    }
}

fn nsstring(s: &str) -> *mut AnyObject {
    use std::ffi::CString;
    let c = CString::new(s).unwrap_or_else(|_| CString::new("").unwrap());
    unsafe { msg_send![class!(NSString), stringWithUTF8String: c.as_ptr()] }
}

unsafe fn activate_application() {
    let ns_app: *mut AnyObject = msg_send![class!(NSApplication), sharedApplication];
    let _: () = msg_send![ns_app, activateIgnoringOtherApps: true];
    let nil: *mut AnyObject = std::ptr::null_mut();
    let _: () = msg_send![ns_app, unhide: nil];
    let _: () = msg_send![ns_app, arrangeInFront: nil];

    let windows: *mut AnyObject = msg_send![ns_app, windows];
    if windows.is_null() {
        return;
    }
    let count: NSUInteger = msg_send![windows, count];
    for i in 0..count {
        let window: *mut AnyObject = msg_send![windows, objectAtIndex: i];
        if window.is_null() {
            continue;
        }
        let is_mini: bool = msg_send![window, isMiniaturized];
        if is_mini {
            let _: () = msg_send![window, deminiaturize: nil];
        }
        let _: () = msg_send![window, makeKeyAndOrderFront: nil];
    }
}

#[cfg(target_pointer_width = "64")]
type CGFloat = f64;

#[cfg(not(target_pointer_width = "64"))]
type CGFloat = f32;

type NSUInteger = usize;
