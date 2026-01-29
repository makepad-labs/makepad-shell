use super::MenuItem;

#[derive(Debug, Clone)]
pub struct MenuBarModel {
    pub menus: Vec<TopMenu>,
}

impl MenuBarModel {
    pub fn new(menus: Vec<TopMenu>) -> Self {
        Self { menus }
    }
}

#[derive(Debug, Clone)]
pub struct TopMenu {
    pub label: String,
    pub items: Vec<MenuItem>,
    pub role: Option<TopMenuRole>,
}

impl TopMenu {
    pub fn new(label: impl Into<String>, items: Vec<MenuItem>) -> Self {
        Self {
            label: label.into(),
            items,
            role: None,
        }
    }

    pub fn with_role(mut self, role: TopMenuRole) -> Self {
        self.role = Some(role);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TopMenuRole {
    App,
    File,
    Edit,
    View,
    Window,
    Help,
}
