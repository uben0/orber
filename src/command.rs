use crate::{SetPlacingBlock, block::Block};
use bevy::ecs::system::Commands;

lalrpop_util::lalrpop_mod!(commands);

pub use commands::CommandParser as UserCommandParser;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Command {
    Test,
    Place(Block),
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
        }
    }
}
