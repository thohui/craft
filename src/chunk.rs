use std::collections::HashMap;

use noise::utils::NoiseMapBuilder;
use noise::NoiseFn;
use noise::{utils::PlaneMapBuilder, Fbm, Perlin};

use crate::noise::generate_perlin_noise;
use crate::renderer::block::{self, Block, BlockType, Face, TerrainMesh};

pub struct Chunk {
    pub position: cgmath::Vector3<f32>,
    blocks: Vec<Vec<Vec<Block>>>,
    mesh: TerrainMesh,
}

const CHUNK_WIDTH: usize = 32;
const CHUNK_HEIGHT: usize = 32;
const CHUNK_DEPTH: usize = 32;

impl Chunk {
    pub fn new(position: cgmath::Vector3<f32>) -> Self {
        let mut this = Self {
            position,
            mesh: TerrainMesh::new(),
            blocks: vec![
                vec![
                    vec![
                        Block::new(BlockType::Air, cgmath::Vector3::new(0.0, 0.0, 0.0));
                        CHUNK_DEPTH as usize
                    ];
                    CHUNK_HEIGHT as usize
                ];
                CHUNK_WIDTH as usize
            ],
        };

        this
    }

    pub fn mesh(&self) -> &TerrainMesh {
        &self.mesh
    }

    fn init(&mut self, height_map: &HashMap<(usize, usize), f32>) {
        let block_size = 2.0;
        for x in 0..CHUNK_WIDTH as usize {
            for z in 0..CHUNK_DEPTH as usize {
                let height_map_x = x + self.position.x as usize;
                let height_map_z = z + self.position.z as usize;

                let terrain_height = *height_map.get(&(height_map_x, height_map_z)).unwrap();

                for y in 0..CHUNK_HEIGHT as usize {
                    let mut block_type = BlockType::Air;

                    if y == terrain_height as usize {
                        block_type = BlockType::Grass;
                    } else if y == 0 {
                        block_type = BlockType::Stone;
                    } else if y < terrain_height as usize {
                        block_type = BlockType::Dirt;
                    }

                    let position = cgmath::Vector3::new(
                        self.position.x + x as f32,
                        self.position.y + y as f32,
                        self.position.z + z as f32,
                    ) * block_size;

                    self.blocks[x][y][z] = Block::new(block_type, position);
                }
            }
        }

        self.generate_mesh();
    }

    pub fn generate_mesh(&mut self) {
        self.mesh = TerrainMesh::new();

        for x in 0..CHUNK_WIDTH {
            for y in 0..CHUNK_HEIGHT {
                for z in 0..CHUNK_DEPTH {
                    let block = &self.blocks[x][y][z];

                    if block.is_air() {
                        continue;
                    }

                    let x = x as isize;
                    let y = y as isize;
                    let z = z as isize;

                    // TODO: check neighbors between chunks.

                    // check left neighbor
                    if self.should_render_face(x - 1, y, z) {
                        self.mesh.add_face(block.generate_face(Face::Left));
                    }
                    // check right neighbor
                    if self.should_render_face(x + 1, y, z) {
                        self.mesh.add_face(block.generate_face(Face::Right));
                    }
                    // check bottom neighbor
                    if self.should_render_face(x, y - 1, z) {
                        self.mesh.add_face(block.generate_face(Face::Bottom));
                    }
                    // check top neighbor
                    if self.should_render_face(x, y + 1, z) {
                        self.mesh.add_face(block.generate_face(Face::Top));
                    }
                    // check front neighbor
                    if self.should_render_face(x, y, z - 1) {
                        self.mesh.add_face(block.generate_face(Face::Front));
                    }
                    // check back neighbor
                    if self.should_render_face(x, y, z + 1) {
                        self.mesh.add_face(block.generate_face(Face::Back));
                    }
                }
            }
        }
    }

    fn should_render_face(&self, x: isize, y: isize, z: isize) -> bool {
        // check out of bounds.
        if x < 0
            || x >= CHUNK_WIDTH as isize
            || y < 0
            || y >= CHUNK_HEIGHT as isize
            || z < 0
            || z >= CHUNK_DEPTH as isize
        {
            return true;
        }

        let block = self.blocks[x as usize][y as usize][z as usize];

        block.is_air()
    }
}

pub fn generate_chunks(chunk_count: usize) -> Vec<Chunk> {
    let scale = 50.0;
    let seed = 1234;

    let height_min = 0.0;
    let height_max = 15.0;

    let block_size = 2.0;
    let height_map = generate_perlin_noise(
        chunk_count * CHUNK_WIDTH as usize,
        chunk_count * CHUNK_DEPTH as usize,
        scale,
        seed,
        height_min,
        height_max,
    );

    let mut chunks = Vec::new();
    for chunk_x in 0..chunk_count {
        for chunk_z in 0..chunk_count {
            chunks.push(Chunk::new(cgmath::Vector3::new(
                chunk_x as f32 * CHUNK_WIDTH as f32,
                0 as f32 * CHUNK_HEIGHT as f32,
                chunk_z as f32 * CHUNK_DEPTH as f32,
            )));
        }
    }

    chunks.iter_mut().for_each(|ch| ch.init(&height_map));
    chunks
}

pub struct ChunkList {
    /// The list of chunks.
    chunks: Vec<Chunk>,
    /// The calculated mesh of all the chunks.
    calculated_mesh: Option<TerrainMesh>,
}

impl ChunkList {
    pub fn new(chunks: Vec<Chunk>) -> Self {
        Self {
            chunks,
            calculated_mesh: None,
        }
    }

    pub fn add_chunk(&mut self, chunk: Chunk) {
        self.chunks.push(chunk);
    }

    pub fn get_chunk(&self, position: cgmath::Vector3<f32>) -> Option<&Chunk> {
        self.chunks.iter().find(|ch| ch.position == position)
    }

    pub fn get_chunk_mut(&mut self, position: cgmath::Vector3<f32>) -> Option<&mut Chunk> {
        self.chunks.iter_mut().find(|ch| ch.position == position)
    }

    pub fn merge_meshes(&mut self) -> TerrainMesh {
        // Merge all the meshes of the chunks into a single mesh.
        let mut global_vertices: Vec<block::BlockVertex> = Vec::new();
        let mut global_indices: Vec<u32> = Vec::new();
        for chunk in self.chunks.iter() {
            let mesh = chunk.mesh();
            let vertices = mesh.vertices();
            let indices = mesh.indices();

            let base_index = global_vertices.len() as u32;

            for vertex in vertices.iter() {
                global_vertices.push(*vertex);
            }

            for index in indices.iter() {
                global_indices.push(base_index + *index);
            }
        }

        let mut mesh = TerrainMesh::new();
        mesh.set_vertices(global_vertices);
        mesh.set_indices(global_indices);

        mesh
    }

    pub fn mesh(&mut self) -> &TerrainMesh {
        if self.calculated_mesh.is_none() {
            self.calculated_mesh = Some(self.merge_meshes());
        }

        self.calculated_mesh.as_ref().unwrap()
    }
}
