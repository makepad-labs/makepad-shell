use std::num::NonZeroU64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CommandId(NonZeroU64);

impl CommandId {
    pub fn new(id: u64) -> Option<Self> {
        NonZeroU64::new(id).map(Self)
    }
    
    pub fn as_u64(&self) -> u64 {
        self.0.get()
    }
}

#[derive(Debug, Clone)]
pub struct MenuModel {
    pub items: Vec<MenuItem>,
}

impl MenuModel {
    pub fn new(items: Vec<MenuItem>) -> Self {
        Self { items }
    }
}

#[derive(Debug, Clone)]
pub enum MenuItem {
    Command(CommandItem),
    Submenu(Submenu),
    Separator,
}

#[derive(Debug, Clone)]
pub struct CommandItem {
    pub id: CommandId,
    pub label: String,
    pub enabled: bool,
    pub checked: bool,
    pub shortcut: Option<crate::menu::shortcut::Shortcut>,
    pub role: Option<MenuItemRole>,
}

impl CommandItem {
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

    pub fn with_role(mut self, role: MenuItemRole) -> Self {
        self.role = Some(role);
        self
    }
}

#[derive(Debug, Clone)]
pub struct Submenu {
    pub label: String,
    pub items: Vec<MenuItem>,
}

impl Submenu {
    pub fn new(label: impl Into<String>, items: Vec<MenuItem>) -> Self {
        Self {
            label: label.into(),
            items,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MenuItemRole {
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
