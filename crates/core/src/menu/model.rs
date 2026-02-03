use crate::command::CommandId;
use crate::shortcut::Shortcut;

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
    pub shortcut: Option<Shortcut>,
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
