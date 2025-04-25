use glam::{
    f32::Vec3,
    Mat4, Quat,
};
use std::{error::Error, sync::{Arc, Mutex}};
use vulkano::{
    buffer::{
        allocator::{SubbufferAllocator, SubbufferAllocatorCreateInfo}, BufferContents, BufferUsage
    }, command_buffer::{
        CommandBufferBeginInfo, CommandBufferLevel, CommandBufferUsage, RecordingCommandBuffer, RenderPassBeginInfo 
    }, descriptor_set::{
        layout::{DescriptorSetLayout, DescriptorSetLayoutCreateFlags, DescriptorSetLayoutCreateInfo, DescriptorType}, DescriptorSet, WriteDescriptorSet
    }, memory::allocator::MemoryTypeFilter, pipeline::{
        graphics::vertex_input::Vertex, Pipeline, PipelineBindPoint,
    }, shader::ShaderStages, swapchain::{
        SwapchainCreateInfo, SwapchainPresentInfo,
    }, sync::{self, GpuFuture}, Validated, VulkanError, swapchain
};
use winit::event::{Event, WindowEvent};
use vertices::Transform;

mod shaders;
mod camera;
mod frames_per_second;
mod utils;
mod supply_demand;
mod user_interface;
mod vertices;
mod polynomial;
mod image;
mod snake;

#[derive(BufferContents, Vertex, Clone, Debug)]
#[repr(C)]
pub struct CubeInput {
    #[format(R32G32B32A32_SFLOAT)] pub transform: [[f32; 4]; 4],
    #[format(R32G32B32_SFLOAT)] pub color: [f32; 3],
}

#[derive(Clone)]
struct Cube {
    transform: Transform,
    color: [f32; 3],
    acceleration: [f32; 3],
}

#[derive(Debug)]
struct Vector {
    position: Vec3,
    vector: Vec3,
}

impl From<&Vector> for CubeInput {
    fn from(val: &Vector) -> Self {
        let scale = Vec3::new(val.vector.length(), 0.1, 0.1); 
        let cross = Vec3::new(1.0, 0.0, 0.0).cross(val.vector).normalize();
        // this is stupid
        if cross.is_nan() { 
            let vector = Vec3::new(val.vector[0], 0.1, 0.1);
            return CubeInput {
                transform: Mat4::from_scale_rotation_translation(
                    vector, Quat::default(), val.position + vector * 0.5).to_cols_array_2d(),
                color: [1.0, 0.0, 0.0],
            }
        }
        let rotation = Quat::from_axis_angle(cross, Vec3::new(1.0, 0.0, 0.0).angle_between(val.vector));

        CubeInput {
            transform: Mat4::from_scale_rotation_translation(
                scale, rotation, val.position + rotation * scale * 0.5).to_cols_array_2d(),
            color: [1.0, 0.0, 0.0],
        }
    }
}

fn slider_value(element: &mut user_interface::Element) -> &mut f32 {
    match &mut element.kind {
        user_interface::ElementKind::Slider { value, .. } => value,
        user_interface::ElementKind::Button { .. } => panic!("wrong gui"),
    }
}

fn block_stack(blocks: &Vec<(f32, [f32; 3])>, transform: &Transform) -> Vec<CubeInput> {
    let mut output = vec![]; 

    let mut negative = 0.0;
    for block in blocks {
        if block.0 < 0.0 { negative += block.0; }
    }

    let mut translation = transform.translation;
    let remaining = blocks.iter().filter(|item| item.0 > 0.0).count();
    if remaining != 0 {
        translation += transform.rotation * Vec3::new(0.0, negative, 0.0) * transform.scale[1];
    }

    for block in blocks.iter().filter(|item| if remaining == 0 { true } else { item.0 > 0.0 }) {
        if block.0 == 0.0 && blocks.len() != 1 { continue; }
        translation += transform.rotation * Vec3::new(0.0, block.0, 0.0) * transform.scale[1] * 0.5;
        output.push(CubeInput {
            transform: Transform {
                scale: Vec3::new(transform.scale[0], transform.scale[1] * block.0, transform.scale[2]),
                rotation: transform.rotation,
                translation,
            }.array_matrix(),
            color: block.1,
        });
        translation += transform.rotation * Vec3::new(0.0, block.0, 0.0) * transform.scale[1] * 0.5;
    }
    output
}

fn main() -> Result<(), impl Error> {
    // should be split into a few smaller setup functions (just use library's utils)
    let (device, memory_allocator, images, render_pass, window, event_loop, mut swapchain, queue, command_buffer_allocator, descriptor_set_allocator) = utils::setup();

    let shader_sets = vec![utils::ShaderSet {
        vs: shaders::vs::load(device.clone()).unwrap().entry_point("main").unwrap(),
        fs: shaders::fs::load(device.clone()).unwrap().entry_point("main").unwrap(),
        input: vec![vertices::Position::per_vertex(), CubeInput::per_instance()],
    }, utils::ShaderSet {
        vs: shaders::vs_supply_demand::load(device.clone()).unwrap().entry_point("main").unwrap(),
        fs: shaders::fs_supply_demand::load(device.clone()).unwrap().entry_point("main").unwrap(),
        input: vec![vertices::UvPosition::per_vertex(), supply_demand::GraphInput::per_instance()], 
    }, utils::ShaderSet {
        vs: shaders::vs_user_interface::load(device.clone()).unwrap().entry_point("main").unwrap(),
        fs: shaders::fs_user_interface::load(device.clone()).unwrap().entry_point("main").unwrap(),
        input: vec![vertices::Position2D::per_vertex(), user_interface::QuadInput::per_instance()], 
    }, utils::ShaderSet {
        vs: shaders::vs_image::load(device.clone()).unwrap().entry_point("main").unwrap(),
        fs: shaders::fs_image::load(device.clone()).unwrap().entry_point("main").unwrap(),
        input: vec![vertices::UvPosition::per_vertex(), image::TransformInput::per_instance()], 
    }];

    let (mut pipelines, mut framebuffers) = utils::window_size_dependent_setup(
        memory_allocator.lock().unwrap().clone(),
        shader_sets.clone(),
        &images,
        render_pass.clone(),
    );
    let mut recreate_swapchain = false;

    let mut uploads = RecordingCommandBuffer::new(
        command_buffer_allocator.clone(),
        queue.queue_family_index(),
        CommandBufferLevel::Primary,
        CommandBufferBeginInfo {
            usage: CommandBufferUsage::OneTimeSubmit,
            ..Default::default()
        },
    )
    .unwrap();

    let sampler = image::get_sampler(&device); 
    let locked_memory_allocator = memory_allocator.lock().unwrap();
    let texture = image::get_texture(&locked_memory_allocator, &mut uploads);
    drop(locked_memory_allocator);
    let image_set = DescriptorSet::new(
        descriptor_set_allocator.clone(),
        DescriptorSetLayout::new(
            device.clone(),
            DescriptorSetLayoutCreateInfo {
                flags: DescriptorSetLayoutCreateFlags::empty(),
                bindings: {
                    let mut bindings = std::collections::BTreeMap::new();
                    let mut sampler_buffer = vulkano::descriptor_set::layout::DescriptorSetLayoutBinding::descriptor_type(DescriptorType::CombinedImageSampler); 
                    sampler_buffer.stages = ShaderStages::FRAGMENT;
                    bindings.insert(0, sampler_buffer);
                    bindings
                },
                ..Default::default()
            }
            ).unwrap(), 
        [WriteDescriptorSet::image_view_sampler(0, texture.clone(), sampler.clone()),],
        [],
        )
        .unwrap();

    // well this seems to be the reason images were previously all black
    // let mut previous_frame_end = Some(sync::now(device.clone()).boxed());
    let mut previous_frame_end = Some(
        uploads
            .end()
            .unwrap()
            .execute(queue.clone())
            .unwrap()
            .boxed(),
    );

    let cubes_locked = Arc::new(Mutex::new(vec![]));
    let mut cubes = cubes_locked.lock().unwrap();
    let size = 1.0;        
    for _ in 0..4 {
        (*cubes).push(Cube {
            transform: Transform {
                translation: Vec3::new(rand::random::<f32>() * 200.0 - 100.0, rand::random::<f32>() * 200.0 - 100.0, rand::random::<f32>() * 200.0 - 100.0),
                scale: Vec3::new(size, size, size),
                ..Default::default()
            },
            color: [0.0, 0.0, 0.0],
            acceleration: [0.0; 3],
        });
    }
    drop(cubes);

    let locked_memory_allocator = memory_allocator.lock().unwrap();
    let uniform_buffer = SubbufferAllocator::new(
        locked_memory_allocator.clone(),
        SubbufferAllocatorCreateInfo {
            buffer_usage: BufferUsage::UNIFORM_BUFFER,
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
    );
    let vertex_buffers = (
        utils::create_buffer(BufferUsage::VERTEX_BUFFER, vertices::CUBE_VERTICES.0, locked_memory_allocator.clone()),
        utils::create_buffer(BufferUsage::VERTEX_BUFFER, vertices::QUAD_VERTICES.0, locked_memory_allocator.clone()),
        utils::create_buffer(BufferUsage::VERTEX_BUFFER, vertices::QUAD_2D_VERTICES.0, locked_memory_allocator.clone()),
    );
    let index_buffers = [
        utils::create_buffer(BufferUsage::INDEX_BUFFER, vertices::CUBE_VERTICES.1, locked_memory_allocator.clone()),
        utils::create_buffer(BufferUsage::INDEX_BUFFER, vertices::QUAD_VERTICES.1, locked_memory_allocator.clone()),
        utils::create_buffer(BufferUsage::INDEX_BUFFER, vertices::QUAD_2D_VERTICES.1, locked_memory_allocator.clone()),
    ];
    drop(locked_memory_allocator);

    let trail = Arc::new(Mutex::new(vec![]));
    let trailing = Arc::new(Mutex::new(false));

    let mut menu_elements = vec![user_interface::Element {
        kind: user_interface::ElementKind::Slider {
            color: [1.0, 1.0, 1.0],
            value: 0.5,
        },
        rectangle: user_interface::Rectangle::new([-1.0, -1.0], [0.2, 0.05]),    
    }, user_interface::Element {
        kind: user_interface::ElementKind::Slider {
            color: [1.0, 0.25, 0.25],
            value: 0.5,
        },
        rectangle: user_interface::Rectangle::new([-1.0, -0.9], [0.2, 0.05]),    
    }, user_interface::Element {
        kind: user_interface::ElementKind::Slider {
            color: [0.25, 1.0, 0.25],
            value: (-0.5_f32.sqrt() + 1.0) / 2.0,
        },
        rectangle: user_interface::Rectangle::new([-1.0, -0.8], [0.2, 0.05]), 
    }, user_interface::Element {
        kind: user_interface::ElementKind::Slider {
            color: [0.25, 0.25, 1.0],
            value: (0.5_f32.sqrt() + 1.0) / 2.0,
        },
        rectangle: user_interface::Rectangle::new([-1.0, -0.7], [0.2, 0.05]),    
    }, user_interface::Element {
        kind: user_interface::ElementKind::Slider {
            color: [0.25, 1.0, 0.25],
            value: 0.5,
        },
        rectangle: user_interface::Rectangle::new([-1.0, -0.4], [0.4, 0.05]),    
    }, user_interface::Element {
        kind: user_interface::ElementKind::Slider {
            color: [1.0, 0.25, 0.25],
            value: 0.5,
        },
        rectangle: user_interface::Rectangle::new([-1.0, -0.3], [0.4, 0.05]),    
    }, user_interface::Element {
        kind: user_interface::ElementKind::Slider {
            color: [0.25, 0.25, 1.0],
            value: 0.0,
        },
        rectangle: user_interface::Rectangle::new([-1.0, -0.2], [0.4, 0.05]),    
    }, user_interface::Element {
        kind: user_interface::ElementKind::Slider {
            color: [1.0, 1.0, 1.0],
            value: 0.5,
        },
        rectangle: user_interface::Rectangle::new([-1.0, -0.1], [0.4, 0.05]),    
    }, user_interface::Element {
        kind: user_interface::ElementKind::Button { color: [1.0, 1.0, 1.0], },
        rectangle: user_interface::Rectangle::new([-1.0, 0.0], [0.1, 0.15]),    
    }];

    
    let snake = Arc::new(Mutex::new(snake::Game {
        size: [100, 100, 100],
        snake: snake::Snake::new(),
        transform: Transform {
            scale: Vec3::new(100.0, 100.0, 100.0),
            ..Default::default()
        },
        food: vec![],
        progress: 0.0,
    }));

    let cubes_locked_counter = Arc::clone(&cubes_locked);
    let trail_counter = Arc::clone(&trail); 
    let trailing_counter = Arc::clone(&trailing);
    let snake_counter = Arc::clone(&snake);
    std::thread::spawn(move || {
        for iteration in 0.. {
            std::thread::sleep(std::time::Duration::from_millis(1));
            let cubes_locked = cubes_locked_counter.lock().unwrap(); 
            let mut cubes = cubes_locked.clone();
            drop(cubes_locked);
            let mut directions = vec![[0.0, 0.0, 0.0]; cubes.len()];
            for i in 0..cubes.len() {
                for i2 in 0..cubes.len() {
                    if i == i2 { continue; }
                    let distance = (
                        (cubes[i].transform.translation[0] + cubes[i].transform.scale[0] / 2.0 - cubes[i2].transform.translation[0] - cubes[i2].transform.scale[0] / 2.0).powi(2) + 
                        (cubes[i].transform.translation[1] + cubes[i].transform.scale[1] / 2.0 - cubes[i2].transform.translation[1] - cubes[i2].transform.scale[1] / 2.0).powi(2) +
                        (cubes[i].transform.translation[2] + cubes[i].transform.scale[2] / 2.0 - cubes[i2].transform.translation[2] - cubes[i2].transform.scale[2] / 2.0).powi(2)
                    ).sqrt();
                    for position in 0..3 {
                        let mut value = cubes[i2].transform.translation[position] - cubes[i].transform.translation[position];
                        value /= distance.powi(2);
                        directions[i][position] += value;
                    }
                }
            }
            for (i, cube) in cubes.iter_mut().enumerate() {
                let constant = 0.0005;
                cube.acceleration[0] += directions[i][0] * constant;
                cube.acceleration[1] += directions[i][1] * constant;
                cube.acceleration[2] += directions[i][2] * constant;
            }
            for cube in &mut cubes {
                cube.transform.translation[0] += cube.acceleration[0];
                cube.transform.translation[1] += cube.acceleration[1];
                cube.transform.translation[2] += cube.acceleration[2];
                let vector = Vec3::from(cube.acceleration);
                cube.transform.rotation = Quat::from_axis_angle(Vec3::new(1.0, 0.0, 0.0).cross(vector).normalize(), Vec3::new(1.0, 0.0, 0.0).angle_between(vector));
            }
            let trailing = trailing_counter.lock().unwrap();
            if iteration % 40 == 0 && *trailing {
                let mut trail = trail_counter.lock().unwrap();
                trail.push(Vector {
                    position: Vec3::new(cubes[0].transform.translation[0], cubes[0].transform.translation[1], cubes[0].transform.translation[2]), 
                    vector: Vec3::new(cubes[0].acceleration[0] * 20.0, cubes[0].acceleration[1] * 20.0, cubes[0].acceleration[2] * 20.0),
                });
            }
            drop(trailing);
            let mut snake_locked = snake_counter.lock().unwrap();
            if iteration % 40 == 0 { snake_locked.run(iteration); }
            snake_locked.progress = (iteration % 40) as f32 / 40.0;
            drop(snake_locked);
            let mut cubes_locked = cubes_locked_counter.lock().unwrap();
            // id system would be useful here huh
            cubes = cubes.drain(0..cubes_locked.len().min(cubes.len())).collect();
            for i in (0..cubes.len()).rev() {
                cubes_locked[i] = cubes.pop().unwrap();
            }
        }
    });

    let mut mouse_escaped = false;
    window.set_cursor_visible(false);
    let mut camera = camera::Camera::new(camera::PureTransform { 
        rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 1.0),
        translation: Vec3::new(0.0, 0.0, 0.0),
    });
    let mut frames_per_second = frames_per_second::FramesPerSecond::new(30);

    let mut following = false;
    let mut following_snake = true;
    let mut cursor_position = [0, 0];
    let mut cursor_pressed = false;
    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => elwt.exit(),
            Event::WindowEvent { event: WindowEvent::Resized(_), .. } => recreate_swapchain = true,
            Event::WindowEvent { event: WindowEvent::KeyboardInput { event, .. }, .. } => {
                if event.state == winit::event::ElementState::Pressed { 
                    let step = 0.1;
                    use winit::keyboard::PhysicalKey::Code;
                    use winit::keyboard::KeyCode;
                    match event.physical_key {
                        Code(KeyCode::KeyW) => camera.go_forward(step),
                        Code(KeyCode::KeyS) => camera.go_forward(-step),
                        Code(KeyCode::KeyD) => camera.go_right(step),
                        Code(KeyCode::KeyA) => camera.go_right(-step),
                        Code(KeyCode::KeyB) => {
                            if following_snake { following_snake = false; return; }
                            if following {
                                following = false;
                                following_snake = true;
                                return;
                            }
                            following = true;
                        },
                        Code(KeyCode::Escape) => {
                            mouse_escaped = ! mouse_escaped;
                            window.set_cursor_visible(mouse_escaped);
                            if ! mouse_escaped { camera::center_cursor(&window, &swapchain); }
                        }
                        Code(KeyCode::KeyN) => {
                            let mut cubes = cubes_locked.lock().unwrap();
                            (*cubes).push(Cube {
                                transform: Transform {
                                    scale: Vec3::new(1.0, 1.0, 1.0),
                                    translation: Vec3::new(
                                        camera.transform.translation[0] * 100.0, 
                                        camera.transform.translation[1] * 100.0, 
                                        camera.transform.translation[2] * 100.0
                                    ),
                                    ..Default::default()
                                },
                                color: [1.0, 0.0, 0.0],
                                acceleration: [0.0; 3],
                            });
                        }
                        Code(KeyCode::KeyR) => {
                            let mut cubes = cubes_locked.lock().unwrap();
                            (*cubes).drain(1..);
                        }
                        Code(KeyCode::KeyM) => {
                            let mut cubes = cubes_locked.lock().unwrap();
                            for _ in 0..25 {
                                let random = [rand::random::<f32>(), rand::random::<f32>(), rand::random::<f32>()]; 
                                (*cubes).push(Cube {
                                    transform: Transform {
                                        scale: Vec3::new(1.0, 1.0, 1.0),
                                        translation: Vec3::new( 
                                            camera.transform.translation[0] * 100.0 + random[0] * 10.0, 
                                            camera.transform.translation[1] * 100.0 + random[1] * 10.0, 
                                            camera.transform.translation[2] * 100.0 + random[2] * 10.0,
                                        ),
                                        ..Default::default()
                                    },
                                    color: random,
                                    acceleration: [0.0; 3],
                                });
                            }
                        }
                        Code(KeyCode::KeyT) => {
                            let mut trailing = trailing.lock().unwrap();
                            *trailing = !*trailing;
                        }
                        Code(KeyCode::KeyC) => {
                            let mut trail = trail.lock().unwrap();
                            *trail = vec![]; 
                        }
                        Code(
                            KeyCode::ArrowRight | KeyCode::ArrowLeft | KeyCode::ArrowUp | KeyCode::ArrowDown |
                            KeyCode::KeyO | KeyCode::KeyL
                        ) => { 
                            let mut snake_locked = snake.lock().unwrap();
                            let vectors = snake_locked.snake.vectors(&Transform::default());
                            let mut best = vectors[0];
                            let camera = match event.physical_key {
                                Code(KeyCode::ArrowRight) => camera.right(),
                                Code(KeyCode::ArrowLeft) => -camera.right(),
                                Code(KeyCode::ArrowUp) => Vec3::new(camera.forward[0], 0.0, camera.forward[2]),
                                Code(KeyCode::ArrowDown) => Vec3::new(-camera.forward[0], 0.0, -camera.forward[2]),
                                Code(KeyCode::KeyO) => Vec3::new(0.0, -1.0, 0.0),
                                Code(KeyCode::KeyL) => Vec3::new(0.0, 1.0, 0.0),
                                _ => unreachable!(),
                            };
                            for vector in &vectors[1..6] {
                                if vector.0.dot(camera) > best.0.dot(camera) {
                                    best = *vector;
                                }
                            } 
                            if best.1 == -snake_locked.snake.direction { return; }
                            snake_locked.snake.set_direction(best.1);
                        }
                        _ => {}
                    }
                }
            }
            Event::WindowEvent { event: WindowEvent::CursorMoved { position, .. }, .. } => {
                if mouse_escaped { 
                    cursor_position = [position.x as u32, position.y as u32];
                    return; 
                }
                let middle = [swapchain.image_extent()[0] / 2, swapchain.image_extent()[1] / 2];
                cursor_position = middle;
                let delta = [position.x - middle[0] as f64, position.y - middle[1] as f64];
                camera::center_cursor(&window, &swapchain);
                let sensitivity = 0.0015;
                camera.turn(-delta[1] as f32 * sensitivity, delta[0] as f32 * sensitivity);
            }
            Event::WindowEvent { event: WindowEvent::MouseInput { state, .. }, .. } => {
                cursor_pressed = match state { 
                    winit::event::ElementState::Pressed => true,        
                    winit::event::ElementState::Released => false,
                };
            }
            Event::WindowEvent { event: WindowEvent::RedrawRequested, .. } => {
                let frame_time = std::time::SystemTime::now();

                if cursor_pressed {
                    let mut values = [0.0; 4];
                    for i in 0..4 {
                        if let user_interface::ElementKind::Slider { value, .. } = menu_elements[i].kind {
                            values[i] = value * 2.0 - 1.0;
                        }
                    }
                    let mut reset = false;
                    for (i, element) in menu_elements.iter_mut().enumerate() {
                        let corner = [
                            ((element.rectangle.corner[0] / 2.0 + 0.5) * swapchain.image_extent()[0] as f32) as u32,
                            ((element.rectangle.corner[1] / 2.0 + 0.5) * swapchain.image_extent()[1] as f32) as u32,
                        ];
                        let size = [
                            (element.rectangle.size[0] * swapchain.image_extent()[0] as f32 / 2.0) as u32,
                            (element.rectangle.size[1] * swapchain.image_extent()[1] as f32 / 2.0) as u32,
                        ];
                        if corner[0] > cursor_position[0] { continue; };
                        if corner[1] > cursor_position[1] { continue; };
                        if corner[0] + size[0] < cursor_position[0] { continue; }
                        if corner[1] + size[1] < cursor_position[1] { continue; }
                        if i < 4 {
                            if let user_interface::ElementKind::Slider { .. } = &mut element.kind {
                                values[i] = (cursor_position[0] - corner[0]) as f32 / size[0] as f32 * 2.0 - 1.0;
                                let mut sum = 0.0;
                                for i2 in 0..4 {
                                    if i2 == i { continue; }
                                    sum += (values[i2]).powi(2);
                                }
                                if sum == 0.0 {
                                    for i2 in 0..4 {
                                        if i2 == i { continue; }
                                        values[i2] = ((1.0 - values[i].powi(2)) / 3.0).sqrt()
                                    }
                                } else {
                                    for i2 in 0..4 {
                                        if i2 == i { continue; }
                                        values[i2] /= (sum / (1.0 - values[i].powi(2))).sqrt();
                                    }
                                }
                            }
                        } 
                        if let user_interface::ElementKind::Slider { value, .. } = &mut element.kind {
                            *value = (cursor_position[0] - corner[0]) as f32 / size[0] as f32;
                        }
                        if i == 8 { reset = true; } 
                    }
                    for i in 0..4 {
                        if let user_interface::ElementKind::Slider { value, .. } = &mut menu_elements[i].kind {
                            *value = (values[i] + 1.0) / 2.0;
                        }
                    }
                    if reset {
                        *slider_value(&mut menu_elements[4]) = 0.5;
                        *slider_value(&mut menu_elements[5]) = 0.5;
                        *slider_value(&mut menu_elements[6]) = 0.0;
                        *slider_value(&mut menu_elements[7]) = 0.5;
                    }
                }

                let locked_cubes = cubes_locked.lock().unwrap();
                let cubes = locked_cubes.clone();
                drop(locked_cubes); 
                if following {
                    camera.transform.translation = Vec3::new(
                        cubes[0].transform.translation[0] / 100.0 - camera.forward[0] / 30.0 + 0.005, 
                        cubes[0].transform.translation[1] / 100.0 - camera.forward[1] / 30.0 + 0.005, 
                        cubes[0].transform.translation[2] / 100.0 - camera.forward[2] / 30.0 + 0.005,
                    );
                }

                let locked_snake = snake.lock().unwrap();
                let snake = locked_snake.clone();
                drop(locked_snake);
                if following_snake {
                    let amount = snake.snake.parts.len();
                    let extended = snake.extend_progress(snake.progress, 
                        snake.snake.parts[amount - 2], snake.snake.parts[amount - 1]);
                    let delta = snake.snake.parts[amount - 1] + -snake.snake.parts[amount - 2];
                    camera.transform.translation = Vec3::new(
                        (extended.translation[0] + extended.scale[0] * 0.5 * delta[0] as f32) / 100.0 - camera.forward[0] / 5.0 + 0.005, 
                        (extended.translation[1] + extended.scale[1] * 0.5 * delta[1] as f32) / 100.0 - camera.forward[1] / 5.0 + 0.005, 
                        (extended.translation[2] + extended.scale[2] * 0.5 * delta[2] as f32) / 100.0 - camera.forward[2] / 5.0 + 0.005,
                    );
                }

                let image_extent: [u32; 2] = window.inner_size().into();
                if image_extent.contains(&0) {return; }

                previous_frame_end.as_mut().unwrap().cleanup_finished();

                if recreate_swapchain {
                    let (new_swapchain, new_images) = swapchain
                        .recreate(SwapchainCreateInfo {
                            image_extent,
                            ..swapchain.create_info()
                        })
                        .expect("failed to recreate swapchain");

                    swapchain = new_swapchain;
                    (pipelines, framebuffers) = utils::window_size_dependent_setup(
                        memory_allocator.lock().unwrap().clone(),
                        shader_sets.clone(),
                        &new_images,
                        render_pass.clone(),
                    );
                    recreate_swapchain = false;
                }

                let uniform_buffer_subbuffer = {
                    let aspect_ratio = swapchain.image_extent()[0] as f32
                        / swapchain.image_extent()[1] as f32;

                    let subbuffer = uniform_buffer.allocate_sized().unwrap();
                    *subbuffer.write().unwrap() = camera.uniform_data(aspect_ratio);

                    subbuffer
                };

                let mut builder = RecordingCommandBuffer::new(
                    command_buffer_allocator.clone(),
                    queue.queue_family_index(),
                    CommandBufferLevel::Primary,
                    CommandBufferBeginInfo {
                        usage: CommandBufferUsage::OneTimeSubmit,
                        ..Default::default()
                    },
                    )
                    .unwrap();

                let (image_index, suboptimal, acquire_future) =
                    match swapchain::acquire_next_image(swapchain.clone(), None).map_err(Validated::unwrap) {
                        Ok(r) => r,
                        Err(VulkanError::OutOfDate) => {
                            recreate_swapchain = true;
                            return;
                        }
                        Err(e) => panic!("failed to acquire next image: {e}"),
                    };
                recreate_swapchain |= suboptimal;

                builder
                    .begin_render_pass(
                        RenderPassBeginInfo {
                            clear_values: vec![
                                Some([0.0, 0.0, 1.0, 1.0].into()),
                                Some(1f32.into()),
                            ],
                            ..RenderPassBeginInfo::framebuffer(
                                framebuffers[image_index as usize].clone(),
                            )
                        },
                        Default::default(),
                        )
                    .unwrap();
                
                let supply_and_demand = supply_demand::Graph {
                    supply: polynomial::Polynomial::new(2.0, 0.0, 0.1),
                    demand: polynomial::Polynomial::new(-1.0, 0.0, 1.0),
                    outside: *slider_value(&mut menu_elements[4]) - 0.5,
                    tax: *slider_value(&mut menu_elements[5]) - 0.5,
                    reduction: *slider_value(&mut menu_elements[6]),
                    slide: *slider_value(&mut menu_elements[7]) - 0.5,
                };

                let view_set = DescriptorSet::new(
                    descriptor_set_allocator.clone(),
                    DescriptorSetLayout::new(
                        device.clone(),
                        DescriptorSetLayoutCreateInfo {
                            flags: DescriptorSetLayoutCreateFlags::empty(),
                            bindings: {
                                let mut bindings = std::collections::BTreeMap::new();
                                let mut uniform_buffer = vulkano::descriptor_set::layout::DescriptorSetLayoutBinding::descriptor_type(DescriptorType::UniformBuffer); 
                                uniform_buffer.stages = ShaderStages::VERTEX;
                                bindings.insert(0, uniform_buffer);
                                bindings
                            },
                            ..Default::default()
                        }
                    ).unwrap(), 
                    [WriteDescriptorSet::buffer(0, uniform_buffer_subbuffer.clone()),],
                    [],
                    )
                    .unwrap();

                let locked_memory_allocator = memory_allocator.lock().unwrap();
                let cube_input_buffer = {
                    let mut cube_inputs = vec![];
                    for cube in &(*cubes) {
                        cube_inputs.push(CubeInput {
                            transform: cube.transform.array_matrix(),
                            color: cube.color,
                        });
                    }
                    let trail = trail.lock().unwrap();
                    for cube in &*trail {
                        cube_inputs.push(cube.into());
                    }
                    let surplus = supply_and_demand.surplus();

                    cube_inputs.append(&mut snake.cubes());

                    let producer_color = [0.9, 0.95, 0.0];
                    let consumer_color = [0.0, 0.75, 1.0];
                    let government_color = [0.05, 0.3, 0.05];
                    let outside_color = [0.1, 0.6, 0.1];
                    let loss_color = [0.9, 0.2, 0.25];

                    for (i, stack) in [
                        vec![
                            (surplus.outside, outside_color), 
                        ], vec![
                            (surplus.outside, outside_color), 
                            (surplus.producer, producer_color), 
                            (surplus.consumer, consumer_color), 
                            (surplus.government, government_color), 
                            (surplus.loss, loss_color), 
                        ], vec![
                            (surplus.producer, producer_color), 
                            (surplus.consumer, consumer_color), 
                            (surplus.government, government_color), 
                        ], 
                        vec![(surplus.producer, producer_color)], 
                        vec![(surplus.consumer, consumer_color)], 
                        vec![(surplus.government, government_color)],
                    ].iter().enumerate() {
                        let transform = Transform {
                            scale: Vec3::new(40.0, 500.0, 40.0),
                            translation: Vec3::new(60.0 * i as f32 - 250.0, 0.0, -100.0),
                            rotation: Quat::from_xyzw(0.0, 0.0, 1.0, 0.0),
                        };
                        for cube in block_stack(stack, &transform) { cube_inputs.push(cube); }
                    }
                    cube_inputs.push(CubeInput {
                        transform: Transform {
                            scale: Vec3::new(500.0, 1.0, 50.0), 
                            translation: Vec3::new(-100.0, 1.0, -100.0),
                            ..Default::default()
                        }.array_matrix(),
                        color: [1.0, 1.0, 1.0],
                    });
                    utils::create_buffer(BufferUsage::VERTEX_BUFFER, cube_inputs, locked_memory_allocator.clone())
                };
                drop(locked_memory_allocator);
                let amount = cube_input_buffer.len();

                builder
                    .bind_pipeline_graphics(pipelines[0].clone()).unwrap()
                    .bind_descriptor_sets(
                        PipelineBindPoint::Graphics,
                        pipelines[0].layout().clone(),
                        0,
                        view_set.clone(), 
                    ).unwrap()
                    .bind_vertex_buffers(0, (vertex_buffers.0.clone(), cube_input_buffer.clone())).unwrap()
                    .bind_index_buffer(index_buffers[0].clone()).unwrap();
                unsafe {
                    builder
                        .draw_indexed(index_buffers[0].len() as u32, amount as u32, 0, 0, 0)
                        .unwrap();
                }

                let locked_memory_allocator = memory_allocator.lock().unwrap();
                let quat_input_buffer = {
                    let quat_inputs = vec![
                        supply_and_demand.input(Transform {
                            scale: Vec3::new(500.0, 500.0, 750.0),
                            rotation: {
                                let mut values = [0.0; 4];
                                for i in 0..4 {
                                    if let user_interface::ElementKind::Slider { value, .. } = menu_elements[i].kind {
                                        values[i] = value;
                                    }
                                }
                                Quat::from_xyzw(
                                    values[1] * 2.0 - 1.0, 
                                    values[2] * 2.0 - 1.0, 
                                    values[3] * 2.0 - 1.0, 
                                    values[0] * 2.0 - 1.0,
                                )
                            },
                            translation: Vec3::new(-100.0, -375.0, 0.0),
                        }),
                    ];
                    utils::create_buffer(BufferUsage::VERTEX_BUFFER, quat_inputs, locked_memory_allocator.clone())
                };
                drop(locked_memory_allocator);
                let amount = quat_input_buffer.len();

                builder
                    .bind_pipeline_graphics(pipelines[1].clone()).unwrap()
                    .bind_descriptor_sets(
                        PipelineBindPoint::Graphics,
                        pipelines[1].layout().clone(),
                        0,
                        view_set.clone(),                                                                                   
                    ).unwrap()
                    .bind_vertex_buffers(0, (vertex_buffers.1.clone(), quat_input_buffer)).unwrap()
                    .bind_index_buffer(index_buffers[1].clone()).unwrap();
                unsafe { 
                    builder
                        .draw_indexed(index_buffers[1].len() as u32, amount as u32, 0, 0, 0)
                        .unwrap();
                }

                let locked_memory_allocator = memory_allocator.lock().unwrap();
                let user_interface_input_buffer = {
                    let mut inputs = vec![];
                    for element in &menu_elements {
                        inputs.append(&mut element.quads());
                    }
                    utils::create_buffer(BufferUsage::VERTEX_BUFFER, inputs, locked_memory_allocator.clone())
                };
                drop(locked_memory_allocator);
                let amount = user_interface_input_buffer.len();

                builder
                    .bind_pipeline_graphics(pipelines[2].clone()).unwrap()
                    .bind_vertex_buffers(0, (vertex_buffers.2.clone(), user_interface_input_buffer)).unwrap()
                    .bind_index_buffer(index_buffers[2].clone()).unwrap();
                unsafe { 
                    builder
                        .draw_indexed(index_buffers[2].len() as u32, if mouse_escaped { amount as u32 } else { 0 }, 0, 0, 0)
                        .unwrap();
                }

                let locked_memory_allocator = memory_allocator.lock().unwrap();
                let image_input_buffer = {
                    let image_inputs = vec![
                        image::TransformInput {
                            transform: Transform {
                                scale: Vec3::new(500.0, 500.0, 500.0),
                                rotation: Quat::from_xyzw(0.0, 1.0, 1.0, 0.0).normalize(),
                                translation: Vec3::new(400.0, -200.0, 0.0),
                            }.array_matrix()
                        } 
                    ];
                    utils::create_buffer(BufferUsage::VERTEX_BUFFER, image_inputs, locked_memory_allocator.clone())
                };
                drop(locked_memory_allocator);

                let uniform_buffer_subbuffer = {
                    let subbuffer = uniform_buffer.allocate_sized().unwrap();
                    *subbuffer.write().unwrap() = crate::shaders::fs_image::FilterData {
                        brightness: *slider_value(&mut menu_elements[0]) * 3.0, 
                    };

                    subbuffer
                };

                let filter_set = DescriptorSet::new(
                    descriptor_set_allocator.clone(),
                    DescriptorSetLayout::new(
                        device.clone(),
                        DescriptorSetLayoutCreateInfo {
                            flags: DescriptorSetLayoutCreateFlags::empty(),
                            bindings: {
                                let mut bindings = std::collections::BTreeMap::new();
                                let mut uniform_buffer = vulkano::descriptor_set::layout::DescriptorSetLayoutBinding::descriptor_type(DescriptorType::UniformBuffer); 
                                uniform_buffer.stages = ShaderStages::FRAGMENT;
                                bindings.insert(0, uniform_buffer);
                                bindings
                            },
                            ..Default::default()
                        }
                        ).unwrap(), 
                    [WriteDescriptorSet::buffer(0, uniform_buffer_subbuffer),],
                    [],
                    )
                        .unwrap();
                builder
                    .bind_pipeline_graphics(pipelines[3].clone()).unwrap()
                    .bind_descriptor_sets(
                        PipelineBindPoint::Graphics,
                        pipelines[3].layout().clone(),
                        0,
                        view_set.clone(),                                                                                   
                    ).unwrap()
                    .bind_descriptor_sets(
                        PipelineBindPoint::Graphics,
                        pipelines[3].layout().clone(),
                        1,
                        image_set.clone(),                                                                                   
                    ).unwrap()
                    .bind_descriptor_sets(
                        PipelineBindPoint::Graphics,
                        pipelines[3].layout().clone(),
                        2,
                        filter_set.clone(),                                                                                   
                    ).unwrap()
                    .bind_vertex_buffers(0, (vertex_buffers.1.clone(), image_input_buffer)).unwrap()
                    .bind_index_buffer(index_buffers[1].clone()).unwrap();
                unsafe { 
                    builder
                        .draw_indexed(index_buffers[1].len() as u32, 1, 0, 0, 0)
                        .unwrap();
                }

                builder.end_render_pass(Default::default()).unwrap();

                let command_buffer = builder.end().unwrap();

                if let Some(output) = frames_per_second.sample() {
                    window.set_title(&format!("{output:.1} fps because of vsync but really it's {:.1} fps", 1_000_000.0 / frame_time.elapsed().unwrap().as_micros() as f64));
                }

                let future = previous_frame_end
                    .take().unwrap()
                    .join(acquire_future)
                    .then_execute(queue.clone(), command_buffer).unwrap()
                    .then_swapchain_present(
                        queue.clone(),
                        SwapchainPresentInfo::swapchain_image_index(swapchain.clone(), image_index),
                        )
                    .then_signal_fence_and_flush();

                match future.map_err(Validated::unwrap) {
                    Ok(future) => {
                        previous_frame_end = Some(future.boxed());
                    }
                    Err(VulkanError::OutOfDate) => {
                        recreate_swapchain = true;
                        previous_frame_end = Some(sync::now(device.clone()).boxed());
                    }
                    Err(e) => {
                        println!("failed to flush future: {e}");
                        previous_frame_end = Some(sync::now(device.clone()).boxed());
                    }
                }
            }
            Event::AboutToWait => window.request_redraw(),
            _ => (),
        }
    })
}
