#[derive(Debug, Clone, Copy)]
pub struct Shortcut {
    pub mods: Modifiers,
    pub key: Key,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum Key {
    Char(char),
    Enter,
    Escape,
    F(u8),
}