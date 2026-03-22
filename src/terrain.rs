use bevy::{
    ecs::resource::Resource,
    math::{IVec2, Vec2},
};

fn sigmoid(x: f32) -> f32 {
    x.exp() / ((-x).exp() + x.exp())
}
fn logistic(x: f32) -> f32 {
    (x.exp() + 1.0).ln()
}
fn terrain(x: f32) -> f32 {
    sigmoid(x) * (1.0 + logistic(x - 6.0))
}

fn harmonic_noise(harmonic: &[(f32, f32)], at: Vec2) -> f32 {
    let mut value = 0.0;
    let mut span = 0.0;
    for &(frequency, amplitude) in harmonic {
        value += amplitude * noisy_bevy::simplex_noise_2d(at / frequency);
        span += amplitude;
    }
    value / span
}

#[derive(Resource)]
pub struct TerrainGenerationParameters {
    noise: Vec<(f32, f32)>,
    continent: f32,
    elevation: f32,
}
impl TerrainGenerationParameters {
    pub fn new() -> Self {
        Self {
            noise: Vec::from([
                (800.0, 8.0),
                (400.0, 4.0),
                (200.0, 2.0),
                (100.0, 1.0),
                (50.0, 0.5),
                (25.0, 0.25),
            ]),
            continent: 8.0,
            elevation: 30.0,
        }
    }
    pub fn descriptor(&self, global: IVec2) -> TerrainDescriptor {
        let continent = harmonic_noise(&self.noise, global.as_vec2());
        let continent = continent * self.continent;
        let elevation = (terrain(continent) - 0.90) * self.elevation;
        let sediment = 1.0;
        TerrainDescriptor {
            continent,
            elevation,
            sediment,
        }
    }
}

pub struct TerrainDescriptor {
    pub continent: f32,
    pub elevation: f32,
    pub sediment: f32,
}
