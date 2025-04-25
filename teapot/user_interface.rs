use vulkano::buffer::BufferContents;
use vulkano::pipeline::graphics::vertex_input::Vertex;

#[derive(Debug)]
pub struct Rectangle {
    pub corner: [f32; 2],
    pub size: [f32; 2],
}

impl Rectangle {
    pub fn new(corner: [f32; 2], size: [f32; 2]) -> Self {
        Self { corner, size, }
    }
}

#[derive(Debug)]
pub enum ElementKind {
    Button {
        color: [f32; 3],
    },
    Slider {
        color: [f32; 3],
        value: f32,
    },
}

#[derive(Debug)]
pub struct Element {
    pub rectangle: Rectangle,
    pub kind: ElementKind,
}

impl Element {
    pub fn quads(&self) -> Vec<QuadInput> {
        match self.kind {
            ElementKind::Button { color, } => {
                vec![
                    QuadInput {
                        translation: self.rectangle.corner,
                        size: self.rectangle.size,
                        color,
                    },
                ]
            }
            ElementKind::Slider { color, value, } => {
                vec![
                    QuadInput {
                        translation: [self.rectangle.corner[0] + self.rectangle.size[0] * value - self.rectangle.size[1] / 2.0, self.rectangle.corner[1]],
                        size: [self.rectangle.size[1]; 2],
                        color: [0.0; 3],
                    },
                    QuadInput {
                        translation: self.rectangle.corner,
                        size: self.rectangle.size,
                        color,
                    },
                ]
            }
        }
    }
}


#[derive(BufferContents, Vertex, Clone)]
#[repr(C)]
pub struct QuadInput {
    #[format(R32G32_SFLOAT)]
    pub translation: [f32; 2],
    #[format(R32G32_SFLOAT)]
    pub size: [f32; 2],
    #[format(R32G32B32_SFLOAT)]
    pub color: [f32; 3],
}

pub mod vs_user_interface {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: r"
            #version 450

            layout(location = 0) in vec2 position;
            layout(location = 1) in vec2 translation;
            layout(location = 2) in vec2 size;
            layout(location = 3) in vec3 color;

            layout(location = 0) out vec3 color_out;

            void main() {
                color_out = color;
                gl_Position = vec4((position + vec2(0.5, 0.5)) * size + translation, 0.0, 1.0);
            }
        ",
    }
}

pub mod fs_user_interface {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: r"
            #version 450

            layout(location = 0) in vec3 color_out;

            layout(location = 0) out vec4 f_color;

            void main() {
                f_color = vec4(color_out, 1.0);
            }
        ",
    }
}
