use crate::command::CommandId;
use crate::shortcut::Shortcut;

#[derive(Clone, Debug)]
pub enum TrayIcon {
    Png { bytes: Vec<u8>, is_template: bool },
}

impl TrayIcon {
    pub fn from_png_bytes(bytes: impl Into<Vec<u8>>) -> Self {
        Self::Png {
            bytes: bytes.into(),
            is_template: false,
        }
    }

    pub fn with_template(mut self, is_template: bool) -> Self {
        let TrayIcon::Png { is_template: flag, .. } = &mut self;
        *flag = is_template;
        self
    }
}

#[derive(Clone, Debug)]
pub struct TrayMenuModel {
    pub items: Vec<TrayMenuItem>,
}

impl TrayMenuModel {
    pub fn new(items: Vec<TrayMenuItem>) -> Self {
        Self { items }
    }
}

#[derive(Clone, Debug)]
pub enum TrayMenuItem {
    Command(TrayCommandItem),
    Submenu(TraySubmenu),
    Separator,
}

#[derive(Clone, Debug)]
pub struct TrayCommandItem {
    pub id: CommandId,
    pub label: String,
    pub enabled: bool,
    pub checked: bool,
    pub shortcut: Option<Shortcut>,
    pub role: Option<TrayMenuItemRole>,
}

impl TrayCommandItem {
    pub fn new(id: CommandId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            enabled: true,
            checked: false,
            shortcut: None,
            role: None,
        }
    }

    pub fn with_role(mut self, role: TrayMenuItemRole) -> Self {
        self.role = Some(role);
        self
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash)]
pub enum TrayMenuItemRole {
    About,
    Preferences,
    Services,
    Hide,
    HideOthers,
    ShowAll,
    Quit,
    Minimize,
    Zoom,
    BringAllToFront,
}

#[derive(Clone, Debug)]
pub struct TraySubmenu {
    pub label: String,
    pub items: Vec<TrayMenuItem>,
}

impl TraySubmenu {
    pub fn new(label: impl Into<String>, items: Vec<TrayMenuItem>) -> Self {
        Self {
            label: label.into(),
            items,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TrayModel {
    pub icon: TrayIcon,
    pub tooltip: Option<String>,
    pub menu: TrayMenuModel,
}

impl TrayModel {
    pub fn new(icon: TrayIcon, menu: TrayMenuModel) -> Self {
        Self {
            icon,
            tooltip: None,
            menu,
        }
    }

    pub fn with_tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }
}
