pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "shaders/vert.glsl",
    }
}
pub mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "shaders/frag.glsl",
    }
}


pub use super::supply_demand::vs_supply_demand;
pub use super::supply_demand::fs_supply_demand;

pub use super::user_interface::vs_user_interface;
pub use super::user_interface::fs_user_interface;

pub use super::image::vs_image;
pub use super::image::fs_image;
