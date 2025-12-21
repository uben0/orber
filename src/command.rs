use crate::{SetPlacingBlock, SetRenderDistance, block::Block};
use bevy::ecs::system::Commands;

lalrpop_util::lalrpop_mod!(commands);

pub use commands::CommandParser as UserCommandParser;

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Test,
    Place(Block),
    RenderDistance(f32),
}

impl Command {
    pub fn dispatch(self, mut commands: Commands) {
        match self {
            Command::Test => {
                println!("test");
            }
            Command::Place(block) => {
                commands.trigger(SetPlacingBlock(block));
            }
            Command::RenderDistance(distance) => {
                commands.trigger(SetRenderDistance(distance));
            }
        }
    }
}
