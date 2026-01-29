use super::model::CommandId;

#[derive(Debug, Clone)]
pub enum MenuEvent {
    CommandInvoked(CommandId),
    Dismissed,
}