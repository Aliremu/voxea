use anyhow::Result;
use image::{load_from_memory_with_format, ImageFormat};
use std::borrow::Cow;
use std::sync::{Arc, OnceLock};
use wgpu::util::{DeviceExt, TextureDataOrder};
use winit::window::Window as WinitWindow;

static mut CONTEXT: OnceLock<RenderContext> = OnceLock::new();

pub fn init() {
    unsafe {
        CONTEXT.get_or_init(|| RenderContext::new());
    }
}

pub fn get() -> &'static RenderContext {
    unsafe { CONTEXT.get().expect("Could not get Render Context!") }
}

pub fn get_mut() -> &'static mut RenderContext {
    unsafe { CONTEXT.get_mut().expect("Could not get Render Context!") }
}

pub struct RenderContext {
    pub(crate) instance: wgpu::Instance,
    pub(crate) adapter: wgpu::Adapter,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) shader: wgpu::ShaderModule,
    pub(crate) renderer: egui_wgpu::Renderer,

    pub(crate) textures: Vec<(wgpu::Texture, wgpu::TextureView, egui::TextureId)>,
}

impl RenderContext {
    pub fn new() -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            ..Default::default()
        });

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            // Request an adapter which can render to our surface
            compatible_surface: None,
        }))
        .expect("Failed to find an appropriate adapter");

        // Create the logical device and command queue
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                required_limits:
                    wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
            },
            None,
        ))
        .expect("Failed to create device");

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        });

        let renderer =
            egui_wgpu::Renderer::new(&device, wgpu::TextureFormat::Bgra8UnormSrgb, None, 1);

        Self {
            instance,
            adapter,
            device,
            queue,
            shader,
            renderer,

            textures: Vec::new(),
        }
    }

    pub fn create_surface(&self, window: Arc<WinitWindow>) -> Result<wgpu::Surface<'static>> {
        let surface = self.instance.create_surface(window)?;

        Ok(surface)
    }

    pub fn create_texture_from_memory(&mut self, data: &[u8]) {
        let png = load_from_memory_with_format(data, ImageFormat::Png).unwrap();
        let texture_desc = wgpu::TextureDescriptor {
            label: None,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            size: wgpu::Extent3d {
                width: png.width(),
                height: png.height(),
                depth_or_array_layers: 1,
            },
            dimension: wgpu::TextureDimension::D2,
            mip_level_count: 1,
            sample_count: 1,
            view_formats: &[wgpu::TextureFormat::Rgba8UnormSrgb],
        };

        let texture = self.device.create_texture_with_data(
            &self.queue,
            &texture_desc,
            TextureDataOrder::LayerMajor,
            png.as_bytes(),
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let id =
            self.renderer
                .register_native_texture(&self.device, &view, wgpu::FilterMode::Linear);

        self.textures.push((texture, view, id));
    }
}
