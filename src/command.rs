use crate::{
    SetRenderDistance,
    block::Block,
    player_control::{SetPlacingBlock, ToggleFlying},
};
use bevy::ecs::system::Commands;

lalrpop_util::lalrpop_mod!(command);

pub use command::UserCommandParser;

#[derive(Debug, Clone, PartialEq)]
pub enum UserCommand {
    Test,
    Place(Block),
    RenderDistance(f32),
    Fly(bool),
}

impl UserCommand {
    pub fn dispatch(self, commands: &mut Commands) {
        match self {
            UserCommand::Test => {
                println!("test");
            }
            UserCommand::Place(block) => {
                commands.trigger(SetPlacingBlock(block));
            }
            UserCommand::RenderDistance(distance) => {
                commands.trigger(SetRenderDistance(distance));
            }
            UserCommand::Fly(fly) => {
                commands.trigger(ToggleFlying(Some(fly)));
            }
        }
    }
}
