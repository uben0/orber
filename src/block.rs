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
}
