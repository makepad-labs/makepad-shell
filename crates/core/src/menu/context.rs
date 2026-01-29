#[derive(Debug, Clone, Copy)]
pub enum MenuTrigger {
    MouseRight,
    TouchLongPress,
    Keyboard,
}

#[derive(Debug, Clone, Copy)]
pub enum MenuAnchor {
    Screen { x: f32, y: f32 },
    Window { x: f32, y: f32 },
}