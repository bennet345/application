use vulkano::buffer::BufferContents;
use vulkano::pipeline::graphics::vertex_input::Vertex;

#[derive(Clone, Debug)]
pub struct Transform {
    pub scale: glam::Vec3,
    pub rotation: glam::Quat,
    pub translation: glam::Vec3,
}

impl Transform {
    pub fn matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }

    pub fn array_matrix(&self) -> [[f32; 4]; 4] {
        self.matrix().to_cols_array_2d()
    }
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            scale: glam::Vec3::new(1.0, 1.0, 1.0),
            rotation: glam::Quat::from_xyzw(0.0, 0.0, 0.0, 1.0),
            translation: glam::Vec3::new(0.0, 0.0, 0.0),
        }
    }
}

#[derive(BufferContents, Vertex, Clone)]
#[repr(C)]
pub struct Position {
    #[format(R32G32B32_SFLOAT)] pub position: [f32; 3],
}

#[derive(BufferContents, Vertex, Clone)]
#[repr(C)]
pub struct UvPosition {
    #[format(R32G32B32_SFLOAT)] position: [f32; 3],
    #[format(R32G32_SFLOAT)] uv: [f32; 2],
}

#[derive(BufferContents, Vertex, Clone)]
#[repr(C)]
pub struct Position2D {
    #[format(R32G32_SFLOAT)] pub position: [f32; 2],
}

pub const QUAD_VERTICES: ([UvPosition; 4], [u32; 6]) = (
    [
        UvPosition { 
            position: [-0.5, 0.0, -0.5],
            uv: [0.0, 0.0],
        },
        UvPosition { 
            position: [0.5, 0.0, -0.5], 
            uv: [1.0, 0.0],
        },
        UvPosition { 
            position: [-0.5, 0.0, 0.5], 
            uv: [0.0, 1.0],
        },
        UvPosition { 
            position: [0.5, 0.0, 0.5], 
            uv: [1.0, 1.0],
        },
    ], [
        0, 1, 2, 
        1, 2, 3,
    ],
);
    

pub const CUBE_VERTICES: ([Position; 8], [u32; 36]) = (
    [
        Position { position: [-0.5, -0.5, -0.5] },
        Position { position: [0.5, -0.5, -0.5] },
        Position { position: [-0.5, 0.5, -0.5] },
        Position { position: [0.5, 0.5, -0.5] },
        Position { position: [-0.5, -0.5, 0.5] },
        Position { position: [0.5, -0.5, 0.5] },
        Position { position: [-0.5, 0.5, 0.5] },
        Position { position: [0.5, 0.5, 0.5] },
    ],
    [
        0, 1, 2, 1, 2, 3,
        4, 5, 6, 5, 6, 7,
        0, 1, 4, 1, 4, 5,
        2, 3, 6, 3, 6, 7,
        0, 2, 6, 0, 4, 6,
        1, 3, 7, 1, 5, 7,
    ],
);

pub const QUAD_2D_VERTICES: ([Position2D; 4], [u32; 6]) = (
    [
        Position2D { position: [-0.5, -0.5], },
        Position2D { position: [0.5, -0.5], },
        Position2D { position: [-0.5, 0.5], },
        Position2D { position: [0.5, 0.5], },
    ], [
        0, 1, 2, 
        1, 2, 3,
    ],
);
