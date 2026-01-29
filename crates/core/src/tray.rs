use crate::menu::MenuModel;

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
pub struct TrayModel {
    pub icon: TrayIcon,
    pub tooltip: Option<String>,
    pub menu: MenuModel,
}

impl TrayModel {
    pub fn new(icon: TrayIcon, menu: MenuModel) -> Self {
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
