use vulkano::{
    buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage}, command_buffer::{
        CopyBufferToImageInfo, RecordingCommandBuffer,
    }, device::Device, format::Format, image::{
        sampler::{Filter, Sampler, SamplerAddressMode, SamplerCreateInfo},
        view::ImageView,
        Image, ImageCreateInfo, ImageType, ImageUsage,
    }, memory::allocator::{AllocationCreateInfo, FreeListAllocator, GenericMemoryAllocator, MemoryTypeFilter}, pipeline::graphics::vertex_input::Vertex, DeviceSize 
};
use std::sync::Arc;

#[derive(BufferContents, Vertex, Clone)]
#[repr(C)]
pub struct TransformInput {
    #[format(R32G32B32A32_SFLOAT)] pub transform: [[f32; 4]; 4],
}

pub fn get_texture(memory_allocator: &Arc<GenericMemoryAllocator<FreeListAllocator>>, uploads: &mut RecordingCommandBuffer) -> Arc<vulkano::image::view::ImageView> {
    let png_bytes = include_bytes!("Grant.png").as_slice();
    let decoder = png::Decoder::new(png_bytes);
    let mut reader = decoder.read_info().unwrap();
    let info = reader.info();
    let extent = [info.width, info.height, 1];

    let upload_buffer = Buffer::new_slice(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::TRANSFER_SRC,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_HOST
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        (info.width * info.height * 4) as DeviceSize,
    )
    .unwrap();

    reader
        .next_frame(&mut upload_buffer.write().unwrap())
        .unwrap();

    let image = Image::new(
        memory_allocator.clone(),
        ImageCreateInfo {
            image_type: ImageType::Dim2d,
            format: Format::R8G8B8A8_SRGB,
            extent,
            usage: ImageUsage::TRANSFER_DST | ImageUsage::SAMPLED,
            ..Default::default()
        },
        AllocationCreateInfo::default(),
    )
    .unwrap();

    uploads
        .copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(
            upload_buffer,
            image.clone(),
        ))
        .unwrap();

    ImageView::new_default(image).unwrap()
}

pub fn get_sampler(device: &Arc<Device>) -> std::sync::Arc<vulkano::image::sampler::Sampler> {
    Sampler::new(
        device.clone(),
        SamplerCreateInfo {
            mag_filter: Filter::Linear,
            min_filter: Filter::Linear,
            address_mode: [SamplerAddressMode::Repeat; 3],
            ..Default::default()
        },
    )
    .unwrap()
}

pub mod vs_image {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: r"
            #version 450

            layout(location = 0) in vec3 position;
            layout(location = 1) in vec2 uv;
            layout(location = 2) in mat4 transform;
            layout(location = 0) out vec2 _uv;

            layout(set = 0, binding = 0) uniform Data {
                mat4 world;
                mat4 view;
                mat4 proj;
            } uniforms;

            void main() {
                _uv = uv;
                mat4 worldview = uniforms.view * uniforms.world;
                // is this order i've been using even correct
                gl_Position = uniforms.proj * worldview * transform * vec4(position, 1.0);
            }
        ",
    }
}

pub mod fs_image {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: r"
            #version 450

            layout(location = 0) in vec2 _uv;
            layout(location = 0) out vec4 f_color;

            layout(set = 1, binding = 0) uniform sampler2D tex;
            layout(set = 2, binding = 0) uniform FilterData {
                float brightness;
            } data;

            void main() {
                vec4 color = texture(tex, _uv);
                if (color[0] + color[1] + color[2] < data.brightness) {
                    f_color = color;
                } else {
                    discard;
                }
            }
        ",
    }
}
