use crate::spacial::{Sides, Symetry2};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd)]
pub enum Block {
    Air,
    Water,
    Leaves,
    Log,
    Sand,
    Grass,
    Stone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Oclusion {
    None,
    Full,
}

impl Block {
    pub const fn oclusion(self) -> Oclusion {
        match self {
            Block::Air | Block::Water | Block::Leaves => Oclusion::None,
            Block::Stone | Block::Sand | Block::Grass | Block::Log => Oclusion::Full,
        }
    }
    pub const fn collides(self) -> bool {
        match self {
            Block::Air | Block::Water => false,
            Block::Stone | Block::Sand | Block::Grass | Block::Log | Block::Leaves => true,
        }
    }
    pub const fn regular_textures(self) -> Option<Sides<(Symetry2, u32)>> {
        match self {
            Block::Air | Block::Water => None,
            Block::Stone => Some(Sides {
                x_pos: (Symetry2::PPP, TEXTURE_STONE),
                x_neg: (Symetry2::NNP, TEXTURE_STONE),
                y_pos: (Symetry2::PPP, TEXTURE_STONE),
                y_neg: (Symetry2::PPP, TEXTURE_STONE),
                z_pos: (Symetry2::NNP, TEXTURE_STONE),
                z_neg: (Symetry2::PPP, TEXTURE_STONE),
            }),
            Block::Sand => Some(Sides::all((Symetry2::PPP, TEXTURE_SAND))),
            Block::Grass => Some(Sides {
                x_pos: (Symetry2::NNP, TEXTURE_GRASS_SIDE),
                x_neg: (Symetry2::PPP, TEXTURE_GRASS_SIDE),
                y_pos: (Symetry2::PPP, TEXTURE_GRASS),
                y_neg: (Symetry2::PPP, TEXTURE_DIRT),
                z_pos: (Symetry2::PPP, TEXTURE_GRASS_SIDE),
                z_neg: (Symetry2::NNP, TEXTURE_GRASS_SIDE),
            }),
            Block::Log => Some(Sides {
                x_pos: (Symetry2::NNP, TEXTURE_LOG_SIDE),
                x_neg: (Symetry2::PPP, TEXTURE_LOG_SIDE),
                y_pos: (Symetry2::PPP, TEXTURE_LOG_TOP),
                y_neg: (Symetry2::PPP, TEXTURE_LOG_TOP),
                z_pos: (Symetry2::PPP, TEXTURE_LOG_SIDE),
                z_neg: (Symetry2::NNP, TEXTURE_LOG_SIDE),
            }),
            Block::Leaves => Some(Sides::all((Symetry2::PPP, TEXTURE_LEAVES))),
        }
    }
}

const TEXTURE_STONE: u32 = 0;
const TEXTURE_DIRT: u32 = 1;
const TEXTURE_GRASS_SIDE: u32 = 2;
const TEXTURE_GRASS: u32 = 3;
const TEXTURE_SAND: u32 = 4;
const TEXTURE_LOG_SIDE: u32 = 5;
const TEXTURE_LEAVES: u32 = 6;
const TEXTURE_LOG_TOP: u32 = 8;
