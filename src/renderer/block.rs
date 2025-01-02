use cgmath::{Vector3, Zero};
use winit::dpi::Position;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]

pub struct BlockVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl BlockVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<BlockVertex>() as wgpu::BufferAddress, // 20 bytes
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

pub struct TerrainMesh {
    vertices: Vec<BlockVertex>,
    indices: Vec<u32>,
}

impl TerrainMesh {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn add_face(&mut self, face: BlockQuad) {
        let base_index = self.vertices.len() as u32;

        self.vertices.extend_from_slice(&face.vertices);

        // 1 face = 2 triangles = 6 indices = 4 vertices
        self.indices.push(base_index + 0);
        self.indices.push(base_index + 1);
        self.indices.push(base_index + 2);

        self.indices.push(base_index + 0);
        self.indices.push(base_index + 2);
        self.indices.push(base_index + 3);
    }

    pub fn vertices(&self) -> &[BlockVertex] {
        &self.vertices
    }

    pub fn vertices_mut(&mut self) -> &mut Vec<BlockVertex> {
        &mut self.vertices
    }

    pub fn set_vertices(&mut self, vertices: Vec<BlockVertex>) {
        self.vertices = vertices;
    }

    pub fn indices(&self) -> &[u32] {
        &self.indices
    }

    pub fn indices_mut(&mut self) -> &mut Vec<u32> {
        &mut self.indices
    }

    pub fn set_indices(&mut self, indices: Vec<u32>) {
        self.indices = indices;
    }
}

pub struct BlockQuad {
    vertices: [BlockVertex; 4],
}

impl BlockQuad {
    pub fn vertices(&self) -> &[BlockVertex; 4] {
        &self.vertices
    }
}

#[inline]
fn combine(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

impl BlockQuad {
    pub fn top(tex_coords: [[f32; 2]; 4], position: [f32; 3]) -> Self {
        Self {
            vertices: [
                BlockVertex {
                    position: combine([-1.0, 1.0, -1.0], position),
                    tex_coords: tex_coords[0],
                },
                BlockVertex {
                    position: combine([1.0, 1.0, -1.0], position),
                    tex_coords: tex_coords[1],
                },
                BlockVertex {
                    position: combine([1.0, 1.0, 1.0], position),
                    tex_coords: tex_coords[2],
                },
                BlockVertex {
                    position: combine([-1.0, 1.0, 1.0], position),
                    tex_coords: tex_coords[3],
                },
            ],
        }
    }

    pub fn bottom(tex_coords: [[f32; 2]; 4], position: [f32; 3]) -> Self {
        Self {
            vertices: [
                BlockVertex {
                    position: combine([-1.0, -1.0, -1.0], position),
                    tex_coords: tex_coords[0],
                },
                BlockVertex {
                    position: combine([1.0, -1.0, -1.0], position),
                    tex_coords: tex_coords[1],
                },
                BlockVertex {
                    position: combine([1.0, -1.0, 1.0], position),
                    tex_coords: tex_coords[2],
                },
                BlockVertex {
                    position: combine([-1.0, -1.0, 1.0], position),
                    tex_coords: tex_coords[3],
                },
            ],
        }
    }

    pub fn left(tex_coords: [[f32; 2]; 4], position: [f32; 3]) -> Self {
        Self {
            vertices: [
                BlockVertex {
                    position: combine([-1.0, -1.0, -1.0], position),
                    tex_coords: tex_coords[0],
                },
                BlockVertex {
                    position: combine([-1.0, 1.0, -1.0], position),
                    tex_coords: tex_coords[1],
                },
                BlockVertex {
                    position: combine([-1.0, 1.0, 1.0], position),
                    tex_coords: tex_coords[2],
                },
                BlockVertex {
                    position: combine([-1.0, -1.0, 1.0], position),
                    tex_coords: tex_coords[3],
                },
            ],
        }
    }

    pub fn right(tex_coords: [[f32; 2]; 4], position: [f32; 3]) -> Self {
        Self {
            vertices: [
                BlockVertex {
                    position: combine([1.0, -1.0, -1.0], position),
                    tex_coords: tex_coords[0],
                },
                BlockVertex {
                    position: combine([1.0, 1.0, -1.0], position),
                    tex_coords: tex_coords[1],
                },
                BlockVertex {
                    position: combine([1.0, 1.0, 1.0], position),
                    tex_coords: tex_coords[2],
                },
                BlockVertex {
                    position: combine([1.0, -1.0, 1.0], position),
                    tex_coords: tex_coords[3],
                },
            ],
        }
    }

    pub fn front(tex_coords: [[f32; 2]; 4], position: [f32; 3]) -> Self {
        Self {
            vertices: [
                BlockVertex {
                    position: combine([-1.0, -1.0, -1.0], position),
                    tex_coords: tex_coords[0],
                },
                BlockVertex {
                    position: combine([1.0, -1.0, -1.0], position),
                    tex_coords: tex_coords[1],
                },
                BlockVertex {
                    position: combine([1.0, 1.0, -1.0], position),
                    tex_coords: tex_coords[2],
                },
                BlockVertex {
                    position: combine([-1.0, 1.0, -1.0], position),
                    tex_coords: tex_coords[3],
                },
            ],
        }
    }

    pub fn back(tex_coords: [[f32; 2]; 4], position: [f32; 3]) -> Self {
        Self {
            vertices: [
                BlockVertex {
                    position: combine([-1.0, -1.0, 1.0], position),
                    tex_coords: tex_coords[0],
                },
                BlockVertex {
                    position: combine([1.0, -1.0, 1.0], position),
                    tex_coords: tex_coords[1],
                },
                BlockVertex {
                    position: combine([1.0, 1.0, 1.0], position),
                    tex_coords: tex_coords[2],
                },
                BlockVertex {
                    position: combine([-1.0, 1.0, 1.0], position),
                    tex_coords: tex_coords[3],
                },
            ],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Block {
    pub block_type: BlockType,
    pub position: cgmath::Vector3<f32>,
}

impl Block {
    pub fn new(block_type: BlockType, position: cgmath::Vector3<f32>) -> Self {
        Self {
            block_type,
            position,
        }
    }

    pub fn is_air(&self) -> bool {
        self.block_type == BlockType::Air
    }

    pub fn generate_face(&self, face: Face) -> BlockQuad {
        match face {
            Face::Top => {
                BlockQuad::top(self.block_type.tex_coords(Face::Top), self.position.into())
            }
            Face::Bottom => BlockQuad::bottom(
                self.block_type.tex_coords(Face::Bottom),
                self.position.into(),
            ),
            Face::Left => {
                BlockQuad::left(self.block_type.tex_coords(Face::Left), self.position.into())
            }
            Face::Right => BlockQuad::right(
                self.block_type.tex_coords(Face::Right),
                self.position.into(),
            ),
            Face::Front => BlockQuad::front(
                self.block_type.tex_coords(Face::Front),
                self.position.into(),
            ),
            Face::Back => {
                BlockQuad::back(self.block_type.tex_coords(Face::Back), self.position.into())
            }
        }
    }
}

const ATLAS_SIZE: f32 = 256.0;
const BLOCK_SIZE: f32 = 16.0;

#[repr(u32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BlockType {
    Dirt,
    Grass,
    Stone,
    Air,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum Face {
    Top,
    Bottom,
    Left,
    Right,
    Front,
    Back,
}

impl BlockType {
    pub fn tex_coords(&self, face: Face) -> [[f32; 2]; 4] {
        let (x, y) = match self {
            BlockType::Grass => match face {
                Face::Top => (0, 0),
                Face::Bottom => (2, 0),
                Face::Left | Face::Right | Face::Front | Face::Back => (3, 0),
            },
            BlockType::Dirt => (2, 0),
            BlockType::Stone => (1, 0),
            BlockType::Air => (3, 0),
        };

        let u_min = x as f32 * BLOCK_SIZE / ATLAS_SIZE;
        let v_min = y as f32 * BLOCK_SIZE / ATLAS_SIZE;
        let u_max = u_min + BLOCK_SIZE / ATLAS_SIZE;
        let v_max = v_min + BLOCK_SIZE / ATLAS_SIZE;

        let mut uv_coords = [
            [u_min, v_min],
            [u_max, v_min],
            [u_max, v_max],
            [u_min, v_max],
        ];

        // Fix uv coordinates for the sides of a block.
        match face {
            Face::Front | Face::Back => uv_coords.rotate_right(2),
            Face::Left | Face::Right => uv_coords.rotate_right(1),
            _ => {}
        }

        uv_coords
    }
}
