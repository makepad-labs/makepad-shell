#![cfg(target_os = "macos")]

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use makepad_shell_core::menu::CommandId;
use makepad_shell_core::notification::{Notification, NotificationSound};
use objc2::ffi::NSInteger;
use objc2::rc::Retained;
use objc2::runtime::{AnyObject, Bool, NSObject};
use objc2::{class, define_class, msg_send, MainThreadMarker, MainThreadOnly};

#[derive(Debug)]
pub enum MacNotificationError {
    Unsupported,
    NotOnMainThread,
}

pub fn show_notification_macos(
    notification: Notification,
    on_command: Box<dyn Fn(CommandId) + 'static>,
) -> Result<(), MacNotificationError> {
    let mtm = main_thread_marker();
    ensure_delegate(mtm)?;

    let identifier = notification
        .identifier
        .unwrap_or_else(|| next_identifier());

    let default_action = notification.default_action;
    let (action_label, action_command) = match notification.action_button {
        Some(button) => (Some(button.label), Some(button.command)),
        None => (None, None),
    };

    if default_action.is_some() || action_command.is_some() {
        NOTIFICATION_MAP.with(|map| {
            map.borrow_mut().insert(
                identifier.clone(),
                NotificationEntry {
                    default_command: default_action,
                    action_command,
                    on_command,
                },
            );
        });
    }

    unsafe {
        let notif: *mut AnyObject = msg_send![class!(NSUserNotification), alloc];
        let notif: *mut AnyObject = msg_send![notif, init];

        let title = nsstring(&notification.title);
        let _: () = msg_send![notif, setTitle: title];

        if let Some(subtitle) = notification.subtitle {
            let subtitle = nsstring(&subtitle);
            let _: () = msg_send![notif, setSubtitle: subtitle];
        }

        if let Some(body) = notification.body {
            let body = nsstring(&body);
            let _: () = msg_send![notif, setInformativeText: body];
        }

        let ident = nsstring(&identifier);
        let _: () = msg_send![notif, setIdentifier: ident];

        if let Some(label) = action_label {
            let _: () = msg_send![notif, setHasActionButton: true];
            let label = nsstring(&label);
            let _: () = msg_send![notif, setActionButtonTitle: label];
        }

        match notification.sound {
            NotificationSound::None => {
                let _: () = msg_send![notif, setSoundName: std::ptr::null::<AnyObject>()];
            }
            NotificationSound::Custom(name) => {
                let name = nsstring(&name);
                let _: () = msg_send![notif, setSoundName: name];
            }
            NotificationSound::Default => {
                // NSUserNotification has no sound unless explicitly set.
                // Leave unset for default behavior.
            }
        }

        let Some(center) = notification_center() else {
            return Err(MacNotificationError::Unsupported);
        };
        let _: () = msg_send![center, deliverNotification: notif];
    }

    Ok(())
}

// ------------------------------
// Internal helpers
// ------------------------------

struct NotificationEntry {
    default_command: Option<CommandId>,
    action_command: Option<CommandId>,
    on_command: Box<dyn Fn(CommandId) + 'static>,
}

thread_local! {
    static NOTIFICATION_DELEGATE: RefCell<Option<Retained<NotificationDelegate>>> = RefCell::new(None);
    static NOTIFICATION_MAP: RefCell<HashMap<String, NotificationEntry>> = RefCell::new(HashMap::new());
}

static NOTIFICATION_COUNTER: AtomicU64 = AtomicU64::new(1);

fn next_identifier() -> String {
    let id = NOTIFICATION_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("makepad-shell.notification.{id}")
}

fn ensure_delegate(mtm: MainThreadMarker) -> Result<(), MacNotificationError> {
    NOTIFICATION_DELEGATE.with(|slot| {
        if slot.borrow().is_some() {
            return Ok(());
        }

        let delegate = NotificationDelegate::new(mtm);
        let delegate_ptr = Retained::as_ptr(&delegate) as *mut AnyObject;
        unsafe {
            let Some(center) = notification_center() else {
                return Err(MacNotificationError::Unsupported);
            };
            let _: () = msg_send![center, setDelegate: delegate_ptr];
        }
        *slot.borrow_mut() = Some(delegate);
        Ok(())
    })
}

fn handle_notification_activation(notification: &AnyObject) {
    let activation: NSInteger = unsafe { msg_send![notification, activationType] };
    let ident_obj: *mut AnyObject = unsafe { msg_send![notification, identifier] };
    let Some(identifier) = nsstring_to_string(ident_obj) else {
        return;
    };

    let entry = NOTIFICATION_MAP.with(|map| map.borrow_mut().remove(&identifier));
    let Some(entry) = entry else {
        return;
    };

    let command = if activation == 2 {
        entry.action_command.or(entry.default_command)
    } else {
        entry.default_command.or(entry.action_command)
    };

    if let Some(cmd) = command {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            (entry.on_command)(cmd);
        }));
    }
}

fn notification_center() -> Option<*mut AnyObject> {
    unsafe {
        let center: *mut AnyObject =
            msg_send![class!(NSUserNotificationCenter), defaultUserNotificationCenter];
        if !center.is_null() {
            return Some(center);
        }

        // Ensure NSApplication exists, then retry.
        let _: *mut AnyObject = msg_send![class!(NSApplication), sharedApplication];
        let center: *mut AnyObject =
            msg_send![class!(NSUserNotificationCenter), defaultUserNotificationCenter];
        if center.is_null() {
            None
        } else {
            Some(center)
        }
    }
}

define_class!(
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    struct NotificationDelegate;

    impl NotificationDelegate {
        #[unsafe(method(userNotificationCenter:shouldPresentNotification:))]
        fn should_present_notification(&self, _center: &AnyObject, _notification: &AnyObject) -> Bool {
            Bool::YES
        }

        #[unsafe(method(userNotificationCenter:didActivateNotification:))]
        fn did_activate_notification(&self, _center: &AnyObject, notification: &AnyObject) {
            handle_notification_activation(notification);
        }
    }
);

impl NotificationDelegate {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm);
        unsafe { msg_send![this, init] }
    }
}

fn main_thread_marker() -> MainThreadMarker {
    MainThreadMarker::new().unwrap_or_else(|| unsafe { MainThreadMarker::new_unchecked() })
}

/// Build an autoreleased NSString* from Rust &str.
fn nsstring(s: &str) -> *mut AnyObject {
    use std::ffi::CString;
    let cstring = CString::new(s).unwrap_or_default();
    unsafe {
        let nsstring: *mut AnyObject = msg_send![class!(NSString), stringWithUTF8String: cstring.as_ptr()];
        nsstring
    }
}

fn nsstring_to_string(ns: *mut AnyObject) -> Option<String> {
    if ns.is_null() {
        return None;
    }
    unsafe {
        let cstr: *const std::os::raw::c_char = msg_send![ns, UTF8String];
        if cstr.is_null() {
            return None;
        }
        let text = std::ffi::CStr::from_ptr(cstr).to_string_lossy().into_owned();
        Some(text)
    }
}
