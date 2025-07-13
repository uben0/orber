use crate::spacial::Sides;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Block {
    Air,
    Stone,
    Sand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Oclusion {
    None,
    Full,
}

impl Block {
    pub const fn oclusion(self) -> Oclusion {
        match self {
            Block::Air => Oclusion::None,
            Block::Stone | Block::Sand => Oclusion::Full,
        }
    }
    pub const fn collides(self) -> bool {
        match self {
            Block::Air => false,
            Block::Stone | Block::Sand => true,
        }
    }
    pub const fn textures(self) -> Option<Sides<u32>> {
        match self {
            Block::Air => None,
            Block::Stone => Some(Sides::all(TEXTURE_STONE)),
            Block::Sand => Some(Sides::all(TEXTURE_SAND)),
        }
    }
}

const TEXTURE_STONE: u32 = 0;
// const TEXTURE_DIRT: u32 = 1;
// const TEXTURE_GRASS_SIDE: u32 = 2;
// const TEXTURE_GRASS: u32 = 3;
const TEXTURE_SAND: u32 = 4;
// const TEXTURE_LOG_SIDE: u32 = 5;
// const TEXTURE_LEAVES: u32 = 6;
// const TEXTURE_WATER: u32 = 7;
