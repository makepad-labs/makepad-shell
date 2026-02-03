use crate::command::CommandId;

#[derive(Debug, Clone)]
pub enum MenuEvent {
    CommandInvoked(CommandId),
    Dismissed,
}
