use crate::menu::CommandId;

#[derive(Debug, Clone)]
pub struct Notification {
    pub title: String,
    pub body: Option<String>,
    pub subtitle: Option<String>,
    pub identifier: Option<String>,
    pub default_action: Option<CommandId>,
    pub action_button: Option<NotificationButton>,
    pub sound: NotificationSound,
}

impl Notification {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            body: None,
            subtitle: None,
            identifier: None,
            default_action: None,
            action_button: None,
            sound: NotificationSound::Default,
        }
    }

    pub fn with_body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn with_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    pub fn with_identifier(mut self, identifier: impl Into<String>) -> Self {
        self.identifier = Some(identifier.into());
        self
    }

    pub fn with_default_action(mut self, command: CommandId) -> Self {
        self.default_action = Some(command);
        self
    }

    pub fn with_action_button(mut self, button: NotificationButton) -> Self {
        self.action_button = Some(button);
        self
    }

    pub fn with_sound(mut self, sound: NotificationSound) -> Self {
        self.sound = sound;
        self
    }
}

#[derive(Debug, Clone)]
pub struct NotificationButton {
    pub label: String,
    pub command: CommandId,
}

impl NotificationButton {
    pub fn new(command: CommandId, label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            command,
        }
    }
}

#[derive(Debug, Clone)]
pub enum NotificationSound {
    Default,
    None,
    Custom(String),
}
