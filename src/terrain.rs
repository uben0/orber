use bevy::math::{IVec2, Vec2};

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

pub struct TerrainDescriptor {
    pub continent: f32,
    pub elevation: f32,
    pub sediment: f32,
}

impl TerrainDescriptor {
    pub fn at(global: IVec2) -> Self {
        let continent = harmonic_noise(
            &[(400.0, 4.0), (200.0, 2.0), (100.0, 1.0)],
            global.as_vec2(),
        );
        let continent = continent * 6.0;
        let elevation = (terrain(continent) - 1.0) * 20.0;
        let sediment = 1.0;
        Self {
            continent,
            elevation,
            sediment,
        }
    }
}
