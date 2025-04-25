#[derive(Copy, Clone)]
pub struct PureTransform {
    pub rotation: glam::Quat,
    pub translation: glam::Vec3,
}

pub struct Camera {
    pub transform: PureTransform,
    pub forward: glam::Vec3,
}

impl Camera {
    pub fn new(transform: PureTransform) -> Self {
        Self {
            transform,
            forward: transform.rotation * glam::Vec3::new(1.0, 0.0, 0.0),
        }
    }

    fn up(&self) -> glam::Vec3 { glam::Vec3::new(0.0, 1.0, 0.0) }
    pub fn right(&self) -> glam::Vec3 { self.forward.cross(self.up()).normalize() }

    pub fn turn(&mut self, pitch: f32, yaw: f32) {
        let rotation = glam::Quat::from_axis_angle(self.right(), -pitch) 
            * glam::Quat::from_axis_angle(self.up(), -yaw); 
        let new_rotation = rotation * self.transform.rotation;
        let new_forward = new_rotation * glam::Vec3::new(1.0, 0.0, 0.0);
        if new_forward.dot(glam::Vec3::new(self.forward[0], 0.0, self.forward[2])) <= 0.0 { return; }
        self.transform.rotation = new_rotation; 
        self.forward = new_forward;
    }

    pub fn go_forward(&mut self, step: f32) { self.transform.translation += self.forward * step; }
    pub fn go_right(&mut self, step: f32) { self.transform.translation += self.right() * step; }

    pub fn view_matrix(&self) -> glam::Mat4 {
        glam::Mat4::look_at_rh(
            self.transform.translation,
            self.transform.translation + self.forward,
            self.up(),
        )
    }

    pub fn uniform_data(&self, aspect_ratio: f32) -> crate::shaders::vs::Data {
        let proj = glam::Mat4::perspective_rh_gl(
            std::f32::consts::FRAC_PI_2,
            aspect_ratio,
            0.01,
            100.0,
        );
        let view = self.view_matrix();                    
        let scale = glam::Mat4::from_scale(glam::Vec3::splat(0.01));

        crate::shaders::vs::Data { 
            world: glam::Mat4::from_mat3(glam::Mat3::from_rotation_y(0.0)).to_cols_array_2d(),
            view: (view * scale).to_cols_array_2d(),
            proj: proj.to_cols_array_2d(),
        }
    }
}

pub fn center_cursor(window: &std::sync::Arc<winit::window::Window>, swapchain: &std::sync::Arc<vulkano::swapchain::Swapchain>) {
    let middle = [swapchain.image_extent()[0] / 2, swapchain.image_extent()[1] / 2];
    window.set_cursor_grab(winit::window::CursorGrabMode::Locked).unwrap();
    window.set_cursor_position(winit::dpi::PhysicalPosition::new(middle[0], middle[1])).unwrap();
    window.set_cursor_grab(winit::window::CursorGrabMode::None).unwrap();
}
