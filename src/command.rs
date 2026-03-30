use crate::{
    SetRenderDistance,
    block::Block,
    player_control::{SetPlacingBlock, Teleport, ToggleFlying},
};
use bevy::prelude::*;

lalrpop_util::lalrpop_mod!(command);

pub use command::UserCommandParser;

#[derive(Debug, Clone, PartialEq)]
pub enum UserCommand {
    Place(Block),
    RenderDistance(f32),
    Fly(bool),
    Teleport(Vec3),
}

impl UserCommand {
    pub fn dispatch(self, player: Entity, commands: &mut Commands) {
        match self {
            Self::Teleport(position) => {
                commands
                    .entity(player)
                    .trigger(|target| Teleport { target, position });
                println!("teleport to {position}");
            }
            Self::Place(block) => {
                commands.trigger(SetPlacingBlock(block));
            }
            Self::RenderDistance(distance) => {
                commands.trigger(SetRenderDistance(distance));
            }
            Self::Fly(fly) => {
                commands.trigger(ToggleFlying(Some(fly)));
            }
        }
    }
}
