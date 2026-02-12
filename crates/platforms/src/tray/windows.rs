#![cfg(target_os = "windows")]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::ffi::c_void;
use std::marker::PhantomData;
use std::ptr::{null, null_mut};
use std::sync::OnceLock;

use makepad_shell_core::command::CommandId;
use makepad_shell_core::shortcut::{Key, Shortcut};
use makepad_shell_core::tray::{TrayCommandItem, TrayIcon, TrayMenuItem, TrayMenuModel, TrayModel};

type BOOL = i32;
type UINT = u32;
type DWORD = u32;
type WORD = u16;
type LONG = i32;
type LONG_PTR = isize;
type LPARAM = isize;
type WPARAM = usize;
type LRESULT = isize;
type ATOM = WORD;
type HWND = *mut c_void;
type HMENU = *mut c_void;
type HICON = *mut c_void;
type HINSTANCE = *mut c_void;
type HCURSOR = *mut c_void;
type HBRUSH = *mut c_void;
type WNDPROC = Option<unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT>;

#[repr(C)]
struct POINT {
    x: LONG,
    y: LONG,
}

#[repr(C)]
struct RECT {
    left: LONG,
    top: LONG,
    right: LONG,
    bottom: LONG,
}

#[repr(C)]
struct WNDCLASSW {
    style: UINT,
    lpfnWndProc: WNDPROC,
    cbClsExtra: i32,
    cbWndExtra: i32,
    hInstance: HINSTANCE,
    hIcon: HICON,
    hCursor: HCURSOR,
    hbrBackground: HBRUSH,
    lpszMenuName: *const u16,
    lpszClassName: *const u16,
}

#[repr(C)]
struct CREATESTRUCTW {
    lpCreateParams: *mut c_void,
    hInstance: HINSTANCE,
    hMenu: HMENU,
    hwndParent: HWND,
    cy: i32,
    cx: i32,
    y: i32,
    x: i32,
    style: LONG,
    lpszName: *const u16,
    lpszClass: *const u16,
    dwExStyle: DWORD,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct GUID {
    data1: u32,
    data2: u16,
    data3: u16,
    data4: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy)]
union NotifyIconDataUnion {
    timeout: UINT,
    version: UINT,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct NOTIFYICONDATAW {
    cbSize: DWORD,
    hWnd: HWND,
    uID: UINT,
    uFlags: UINT,
    uCallbackMessage: UINT,
    hIcon: HICON,
    szTip: [u16; 128],
    dwState: DWORD,
    dwStateMask: DWORD,
    szInfo: [u16; 256],
    anonymous: NotifyIconDataUnion,
    szInfoTitle: [u16; 64],
    dwInfoFlags: DWORD,
    guidItem: GUID,
    hBalloonIcon: HICON,
}

const TRUE: BOOL = 1;
const GWLP_USERDATA: i32 = -21;
const MF_STRING: UINT = 0x0000;
const MF_GRAYED: UINT = 0x0001;
const MF_CHECKED: UINT = 0x0008;
const MF_POPUP: UINT = 0x0010;
const MF_SEPARATOR: UINT = 0x0800;
const TPM_RIGHTBUTTON: UINT = 0x0002;
const TPM_RETURNCMD: UINT = 0x0100;
const WM_NULL: UINT = 0x0000;
const WM_CREATE: UINT = 0x0001;
const WM_CONTEXTMENU: UINT = 0x007B;
const WM_LBUTTONDOWN: UINT = 0x0201;
const WM_LBUTTONUP: UINT = 0x0202;
const WM_LBUTTONDBLCLK: UINT = 0x0203;
const WM_RBUTTONUP: UINT = 0x0205;
const WM_APP: UINT = 0x8000;
const NIM_ADD: DWORD = 0x0000;
const NIM_MODIFY: DWORD = 0x0001;
const NIM_DELETE: DWORD = 0x0002;
const NIF_MESSAGE: UINT = 0x0001;
const NIF_ICON: UINT = 0x0002;
const NIF_TIP: UINT = 0x0004;
const IDI_APPLICATION: usize = 32512;

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetModuleHandleW(lpModuleName: *const u16) -> HINSTANCE;
}

#[link(name = "user32")]
unsafe extern "system" {
    fn RegisterClassW(lpWndClass: *const WNDCLASSW) -> ATOM;
    fn CreateWindowExW(
        dwExStyle: DWORD,
        lpClassName: *const u16,
        lpWindowName: *const u16,
        dwStyle: DWORD,
        x: i32,
        y: i32,
        nWidth: i32,
        nHeight: i32,
        hWndParent: HWND,
        hMenu: HMENU,
        hInstance: HINSTANCE,
        lpParam: *mut c_void,
    ) -> HWND;
    fn DefWindowProcW(hWnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn DestroyWindow(hWnd: HWND) -> BOOL;
    fn SetWindowLongPtrW(hWnd: HWND, nIndex: i32, dwNewLong: LONG_PTR) -> LONG_PTR;
    fn GetWindowLongPtrW(hWnd: HWND, nIndex: i32) -> LONG_PTR;
    fn CreatePopupMenu() -> HMENU;
    fn AppendMenuW(hMenu: HMENU, uFlags: UINT, uIDNewItem: usize, lpNewItem: *const u16) -> BOOL;
    fn DestroyMenu(hMenu: HMENU) -> BOOL;
    fn TrackPopupMenu(
        hMenu: HMENU,
        uFlags: UINT,
        x: i32,
        y: i32,
        nReserved: i32,
        hWnd: HWND,
        prcRect: *const RECT,
    ) -> UINT;
    fn SetForegroundWindow(hWnd: HWND) -> BOOL;
    fn GetCursorPos(lpPoint: *mut POINT) -> BOOL;
    fn PostMessageW(hWnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> BOOL;
    fn CreateIconFromResourceEx(
        presbits: *mut u8,
        dwResSize: DWORD,
        fIcon: BOOL,
        dwVer: DWORD,
        cxDesired: i32,
        cyDesired: i32,
        flags: UINT,
    ) -> HICON;
    fn LoadIconW(hInstance: HINSTANCE, lpIconName: *const u16) -> HICON;
    fn DestroyIcon(hIcon: HICON) -> BOOL;
}

#[link(name = "shell32")]
unsafe extern "system" {
    fn Shell_NotifyIconW(dwMessage: DWORD, lpData: *mut NOTIFYICONDATAW) -> BOOL;
}

#[derive(Debug)]
pub enum WindowsTrayError {
    Unsupported,
    BadIcon,
    BadCommandId,
    RegisterClassFailed,
}

const TRAY_WINDOW_CLASS: &str = "MakepadShellTrayWindow";
const TRAY_ICON_ID: u32 = 1;
const WM_TRAY_ICON: u32 = WM_APP + 0x240;

static TRAY_WINDOW_CLASS_REGISTERED: OnceLock<bool> = OnceLock::new();
static TRAY_WINDOW_CLASS_WIDE: OnceLock<Vec<u16>> = OnceLock::new();

pub struct WindowsTrayHandle {
    runtime: *mut WindowsTrayRuntime,
    _not_send_or_sync: PhantomData<std::rc::Rc<()>>,
}

struct WindowsTrayRuntime {
    hwnd: HWND,
    menu: HMENU,
    icon: HICON,
    on_command: Box<dyn Fn(CommandId) + 'static>,
    on_activate: Box<dyn Fn() + 'static>,
}

impl WindowsTrayHandle {
    pub fn update_menu(&mut self, menu: &TrayMenuModel) -> Result<(), WindowsTrayError> {
        let new_menu = build_hmenu(&menu.items)?;
        let runtime = self.runtime_mut()?;
        if !runtime.menu.is_null() {
            unsafe {
                let _ = DestroyMenu(runtime.menu);
            }
        }
        runtime.menu = new_menu;
        Ok(())
    }

    pub fn update_icon(&mut self, icon: &TrayIcon) -> Result<(), WindowsTrayError> {
        let new_icon = build_hicon(icon)?;
        self.runtime_mut()?.set_tray_icon(new_icon)
    }

    pub fn update_tooltip(&mut self, tooltip: Option<&str>) -> Result<(), WindowsTrayError> {
        self.runtime_mut()?.set_tray_tooltip(tooltip)
    }

    fn runtime_mut(&mut self) -> Result<&mut WindowsTrayRuntime, WindowsTrayError> {
        if self.runtime.is_null() {
            return Err(WindowsTrayError::Unsupported);
        }
        unsafe { Ok(&mut *self.runtime) }
    }
}

impl Drop for WindowsTrayHandle {
    fn drop(&mut self) {
        if self.runtime.is_null() {
            return;
        }

        unsafe {
            let mut runtime = Box::from_raw(self.runtime);
            runtime.remove_tray_icon();
            if !runtime.hwnd.is_null() {
                let _ = SetWindowLongPtrW(runtime.hwnd, GWLP_USERDATA, 0);
                let _ = DestroyWindow(runtime.hwnd);
            }
            runtime.destroy_resources();
        }

        self.runtime = null_mut();
    }
}

impl WindowsTrayRuntime {
    fn add_tray_icon(&self, tooltip: Option<&str>) -> Result<(), WindowsTrayError> {
        let mut data = base_notify_data(self.hwnd);
        data.uFlags = NIF_MESSAGE | NIF_ICON;
        data.uCallbackMessage = WM_TRAY_ICON;
        data.hIcon = self.icon;
        if let Some(text) = tooltip {
            data.uFlags |= NIF_TIP;
            write_wide_z(&mut data.szTip, text);
        }

        unsafe {
            if Shell_NotifyIconW(NIM_ADD, &mut data) == 0 {
                return Err(WindowsTrayError::Unsupported);
            }
        }
        Ok(())
    }

    fn set_tray_icon(&mut self, icon: HICON) -> Result<(), WindowsTrayError> {
        let mut data = base_notify_data(self.hwnd);
        data.uFlags = NIF_ICON;
        data.hIcon = icon;

        unsafe {
            if Shell_NotifyIconW(NIM_MODIFY, &mut data) == 0 {
                let _ = DestroyIcon(icon);
                return Err(WindowsTrayError::Unsupported);
            }
            if !self.icon.is_null() {
                let _ = DestroyIcon(self.icon);
            }
        }

        self.icon = icon;
        Ok(())
    }

    fn set_tray_tooltip(&self, tooltip: Option<&str>) -> Result<(), WindowsTrayError> {
        let mut data = base_notify_data(self.hwnd);
        data.uFlags = NIF_TIP;
        if let Some(text) = tooltip {
            write_wide_z(&mut data.szTip, text);
        }

        unsafe {
            if Shell_NotifyIconW(NIM_MODIFY, &mut data) == 0 {
                return Err(WindowsTrayError::Unsupported);
            }
        }

        Ok(())
    }

    fn remove_tray_icon(&self) {
        let mut data = base_notify_data(self.hwnd);
        unsafe {
            let _ = Shell_NotifyIconW(NIM_DELETE, &mut data);
        }
    }

    fn destroy_resources(&mut self) {
        unsafe {
            if !self.menu.is_null() {
                let _ = DestroyMenu(self.menu);
                self.menu = null_mut();
            }
            if !self.icon.is_null() {
                let _ = DestroyIcon(self.icon);
                self.icon = null_mut();
            }
        }
    }

    fn handle_tray_message(&self, event: u32) {
        match event {
            WM_LBUTTONDOWN | WM_LBUTTONUP | WM_LBUTTONDBLCLK => {
                (self.on_activate)();
            }
            WM_RBUTTONUP | WM_CONTEXTMENU => {
                self.show_menu();
            }
            _ => {}
        }
    }

    fn show_menu(&self) {
        unsafe {
            if self.menu.is_null() || self.hwnd.is_null() {
                return;
            }

            let mut point = POINT { x: 0, y: 0 };
            if GetCursorPos(&mut point) == 0 {
                return;
            }

            let _ = SetForegroundWindow(self.hwnd);
            let selected = TrackPopupMenu(
                self.menu,
                TPM_RETURNCMD | TPM_RIGHTBUTTON,
                point.x,
                point.y,
                0,
                self.hwnd,
                null(),
            ) as u32;

            if selected != 0 {
                if let Some(command) = CommandId::new(selected as u64) {
                    (self.on_command)(command);
                }
            }

            let _ = PostMessageW(self.hwnd, WM_NULL, 0, 0);
        }
    }
}

pub fn create_tray_windows(
    model: TrayModel,
    on_command: Box<dyn Fn(CommandId) + 'static>,
    on_activate: Box<dyn Fn() + 'static>,
) -> Result<WindowsTrayHandle, WindowsTrayError> {
    ensure_window_class_registered()?;

    let menu = build_hmenu(&model.menu.items)?;
    let icon = build_hicon(&model.icon)?;

    let runtime = Box::new(WindowsTrayRuntime {
        hwnd: null_mut(),
        menu,
        icon,
        on_command,
        on_activate,
    });
    let runtime_ptr = Box::into_raw(runtime);

    let class_name = tray_window_class_wide();
    let hinstance = unsafe { GetModuleHandleW(null()) };
    let hwnd = unsafe {
        CreateWindowExW(
            0,
            class_name.as_ptr(),
            class_name.as_ptr(),
            0,
            0,
            0,
            0,
            0,
            null_mut(),
            null_mut(),
            hinstance,
            runtime_ptr.cast(),
        )
    };

    if hwnd.is_null() {
        unsafe {
            let mut runtime = Box::from_raw(runtime_ptr);
            runtime.destroy_resources();
        }
        return Err(WindowsTrayError::Unsupported);
    }

    let add_result = unsafe { (&*runtime_ptr).add_tray_icon(model.tooltip.as_deref()) };
    if let Err(err) = add_result {
        unsafe {
            let _ = SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            let _ = DestroyWindow(hwnd);
            let mut runtime = Box::from_raw(runtime_ptr);
            runtime.destroy_resources();
        }
        return Err(err);
    }

    Ok(WindowsTrayHandle {
        runtime: runtime_ptr,
        _not_send_or_sync: PhantomData,
    })
}

unsafe extern "system" fn tray_window_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_CREATE {
        let create = lparam as *const CREATESTRUCTW;
        if create.is_null() {
            return 0;
        }
        let runtime_ptr = unsafe { (*create).lpCreateParams as *mut WindowsTrayRuntime };
        if runtime_ptr.is_null() {
            return 0;
        }
        unsafe {
            (*runtime_ptr).hwnd = hwnd;
            let _ = SetWindowLongPtrW(hwnd, GWLP_USERDATA, runtime_ptr as LONG_PTR);
        }
        return 0;
    }

    let runtime_ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WindowsTrayRuntime };
    if runtime_ptr.is_null() {
        return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
    }

    if msg == WM_TRAY_ICON {
        unsafe {
            (*runtime_ptr).handle_tray_message(lparam as u32);
        }
        return 0;
    }

    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

fn ensure_window_class_registered() -> Result<(), WindowsTrayError> {
    let registered = TRAY_WINDOW_CLASS_REGISTERED.get_or_init(|| unsafe {
        let class_name = tray_window_class_wide();
        let hinstance = GetModuleHandleW(null());

        let window_class = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(tray_window_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance,
            hIcon: null_mut(),
            hCursor: null_mut(),
            hbrBackground: null_mut(),
            lpszMenuName: null(),
            lpszClassName: class_name.as_ptr(),
        };

        RegisterClassW(&window_class) != 0
    });

    if *registered {
        Ok(())
    } else {
        Err(WindowsTrayError::RegisterClassFailed)
    }
}

fn build_hmenu(items: &[TrayMenuItem]) -> Result<HMENU, WindowsTrayError> {
    let menu = unsafe { CreatePopupMenu() };
    if menu.is_null() {
        return Err(WindowsTrayError::Unsupported);
    }

    for item in items {
        if let Err(err) = append_menu_item(menu, item) {
            unsafe {
                let _ = DestroyMenu(menu);
            }
            return Err(err);
        }
    }

    Ok(menu)
}

fn append_menu_item(menu: HMENU, item: &TrayMenuItem) -> Result<(), WindowsTrayError> {
    match item {
        TrayMenuItem::Separator => unsafe {
            if AppendMenuW(menu, MF_SEPARATOR, 0, null()) == 0 {
                return Err(WindowsTrayError::Unsupported);
            }
        },
        TrayMenuItem::Command(command) => {
            let menu_id = command_id_to_menu_id(command.id)? as usize;
            let mut flags = MF_STRING;
            if !command.enabled {
                flags |= MF_GRAYED;
            }
            if command.checked {
                flags |= MF_CHECKED;
            }

            let label = menu_label(command);
            let wide = wide_null(&label);
            unsafe {
                if AppendMenuW(menu, flags, menu_id, wide.as_ptr()) == 0 {
                    return Err(WindowsTrayError::Unsupported);
                }
            }
        }
        TrayMenuItem::Submenu(submenu) => {
            let nested = build_hmenu(&submenu.items)?;
            let wide = wide_null(&submenu.label);
            unsafe {
                if AppendMenuW(menu, MF_POPUP | MF_STRING, nested as usize, wide.as_ptr()) == 0 {
                    let _ = DestroyMenu(nested);
                    return Err(WindowsTrayError::Unsupported);
                }
            }
        }
    }
    Ok(())
}

fn menu_label(item: &TrayCommandItem) -> String {
    let mut label = item.label.clone();
    if let Some(shortcut) = item.shortcut {
        let text = shortcut_label(shortcut);
        if !text.is_empty() {
            label.push('\t');
            label.push_str(&text);
        }
    }
    label
}

fn shortcut_label(shortcut: Shortcut) -> String {
    let mut parts: Vec<String> = Vec::new();
    if shortcut.mods.ctrl {
        parts.push("Ctrl".to_string());
    }
    if shortcut.mods.alt {
        parts.push("Alt".to_string());
    }
    if shortcut.mods.shift {
        parts.push("Shift".to_string());
    }
    if shortcut.mods.meta {
        parts.push("Win".to_string());
    }

    let key = match shortcut.key {
        Key::Char(ch) => ch.to_ascii_uppercase().to_string(),
        Key::Enter => "Enter".to_string(),
        Key::Escape => "Esc".to_string(),
        Key::F(n) => format!("F{n}"),
    };

    parts.push(key);
    parts.join("+")
}

fn command_id_to_menu_id(id: CommandId) -> Result<u32, WindowsTrayError> {
    let raw = id.as_u64();
    if raw > u32::MAX as u64 {
        return Err(WindowsTrayError::BadCommandId);
    }
    Ok(raw as u32)
}

fn build_hicon(icon: &TrayIcon) -> Result<HICON, WindowsTrayError> {
    match icon {
        TrayIcon::Png { bytes, .. } => unsafe {
            if !bytes.is_empty() {
                let ptr = bytes.as_ptr() as *mut u8;
                let icon =
                    CreateIconFromResourceEx(ptr, bytes.len() as DWORD, TRUE, 0x0003_0000, 0, 0, 0);
                if !icon.is_null() {
                    return Ok(icon);
                }
            }

            // Fallback to a default system icon when bytes are not a Windows icon resource.
            let fallback = LoadIconW(null_mut(), IDI_APPLICATION as *const u16);
            if fallback.is_null() {
                return Err(WindowsTrayError::BadIcon);
            }
            Ok(fallback)
        },
    }
}

fn base_notify_data(hwnd: HWND) -> NOTIFYICONDATAW {
    NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: TRAY_ICON_ID,
        uFlags: 0,
        uCallbackMessage: 0,
        hIcon: null_mut(),
        szTip: [0; 128],
        dwState: 0,
        dwStateMask: 0,
        szInfo: [0; 256],
        anonymous: NotifyIconDataUnion { version: 0 },
        szInfoTitle: [0; 64],
        dwInfoFlags: 0,
        guidItem: GUID::default(),
        hBalloonIcon: null_mut(),
    }
}

fn tray_window_class_wide() -> &'static [u16] {
    TRAY_WINDOW_CLASS_WIDE
        .get_or_init(|| wide_null(TRAY_WINDOW_CLASS))
        .as_slice()
}

fn wide_null(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

fn write_wide_z(dst: &mut [u16], src: &str) {
    for slot in dst.iter_mut() {
        *slot = 0;
    }
    let max = dst.len().saturating_sub(1);
    for (slot, ch) in dst.iter_mut().take(max).zip(src.encode_utf16()) {
        *slot = ch;
    }
}
