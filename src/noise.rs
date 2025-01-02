use std::collections::HashMap;

use noise::{NoiseFn, Perlin};
use rand::Rng;

pub fn generate_perlin_noise(
    chunk_width: usize,
    chunk_depth: usize,
    scale: f64,
    seed: u32,
    height_min: f32,
    height_max: f32,
) -> HashMap<(usize, usize), f32> {
    let mut height_map = HashMap::new();
    let perlin = Perlin::new(seed);

    for x in 0..chunk_width {
        for z in 0..chunk_depth {
            let noise_value = perlin.get([x as f64 / scale, z as f64 / scale]);

            let normalized_height = (noise_value + 1.0) * 0.5;

            let terrain_height = height_min + normalized_height as f32 * (height_max - height_min);

            height_map.insert((x, z), terrain_height);
        }
    }
    height_map
}
