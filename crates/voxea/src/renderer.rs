use log::{info, warn};
use std::borrow::Cow;
use std::sync::Arc;
use winit::dpi::PhysicalSize;
use winit::window::Window as WinitWindow;

pub struct RenderContext {
    pub(crate) instance: wgpu::Instance,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) config: wgpu::SurfaceConfiguration,
    pub(crate) surface: wgpu::Surface<'static>,
    pub(crate) render_pipeline: wgpu::RenderPipeline,
    pub(crate) renderer: egui_wgpu::Renderer,
    pub(crate) egui_state: egui_winit::State,
    pub(crate) ui_cmds: Vec<Box<dyn Fn(&egui::Context) + 'static>>,

    pub(crate) fbo: wgpu::Texture,
    pub(crate) fbo_view: wgpu::TextureView,
    pub(crate) fbo_id: egui::TextureId,
}

impl RenderContext {
    pub fn new(egui_ctx: egui::Context, window: Arc<WinitWindow>) -> Self {
        let inner_size = window.inner_size().max([1, 1].into());
        let width = inner_size.width;
        let height = inner_size.height;

        let egui_state = egui_winit::State::new(
            egui_ctx,
            egui::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            window.theme(),
            None,
        );

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            ..Default::default()
        });
        let surface = instance.create_surface(window).unwrap();

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            // Request an adapter which can render to our surface
            compatible_surface: Some(&surface),
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

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                compilation_options: Default::default(),
                targets: &[Some(swapchain_format.into())],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let mut config = surface.get_default_config(&adapter, width, height).unwrap();
        info!("{:?}", config);

        surface.configure(&device, &config);

        let mut renderer =
            egui_wgpu::Renderer::new(&device, wgpu::TextureFormat::Bgra8UnormSrgb, None, 1);

        let fbo = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("FBO Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[wgpu::TextureFormat::Bgra8UnormSrgb],
        });

        let fbo_view = fbo.create_view(&wgpu::TextureViewDescriptor::default());
        let fbo_id = renderer.register_native_texture(&device, &fbo_view, wgpu::FilterMode::Linear);

        Self {
            instance,
            device,
            queue,
            config,
            surface,
            render_pipeline,
            renderer,
            egui_state,
            ui_cmds: Vec::new(),

            fbo,
            fbo_view,
            fbo_id,
        }
    }

    pub fn resize(&mut self, new_size: &PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn ui<F>(&mut self, f: F)
    where
        F: Fn(&egui::Context) + 'static,
    {
        self.ui_cmds.push(Box::new(f));
    }

    pub fn ui2<F>(&mut self, window: &WinitWindow, mut f: F)
    where
        F: FnMut(&egui::Context),
    {
        let input = self.egui_state.take_egui_input(&window);
        let cx = self.egui_state.egui_ctx();
        let full_output = self.egui_state.egui_ctx().run(input, |cx| {
            f(cx);
        });

        self.ui_cmds.clear();

        let clipped_primitives =
            cx.tessellate(full_output.shapes.clone(), full_output.pixels_per_point);

        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer
                .update_texture(&self.device, &self.queue, *id, image_delta);
        }

        let desc = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: cx.pixels_per_point(),
        };

        let Ok(frame) = self.surface.get_current_texture() else {
            // warn!("Failed to acquire next swap chain texture");
            return;
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let _command_buffers = self.renderer.update_buffers(
            &self.device,
            &self.queue,
            &mut encoder,
            &clipped_primitives,
            &desc,
        );

        // Renders egui interface on top of framebuffer
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            self.renderer.render(&mut rpass, &clipped_primitives, &desc);
        }

        for id in &full_output.textures_delta.free {
            self.renderer.free_texture(id);
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    pub fn draw_triangle(&mut self) {
        let fbo_view = &self.fbo_view;

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // Renders egui interface on top of framebuffer
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: fbo_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            rpass.set_pipeline(&self.render_pipeline);
            rpass.draw(0..3, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));
    }

    pub fn render(&mut self, window: &WinitWindow) {
        let input = self.egui_state.take_egui_input(&window);
        let cx = self.egui_state.egui_ctx();
        let full_output = self.egui_state.egui_ctx().run(input, |cx| {
            for f in &self.ui_cmds {
                f(cx);
            }
        });

        self.ui_cmds.clear();

        let clipped_primitives =
            cx.tessellate(full_output.shapes.clone(), full_output.pixels_per_point);

        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer
                .update_texture(&self.device, &self.queue, *id, image_delta);
        }

        let desc = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: cx.pixels_per_point(),
        };

        let Ok(frame) = self.surface.get_current_texture() else {
            warn!("Failed to acquire next swap chain texture");
            return;
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let _command_buffers = self.renderer.update_buffers(
            &self.device,
            &self.queue,
            &mut encoder,
            &clipped_primitives,
            &desc,
        );

        // Renders egui interface on top of framebuffer
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            rpass.set_pipeline(&self.render_pipeline);
            rpass.draw(0..3, 0..1);

            self.renderer.render(&mut rpass, &clipped_primitives, &desc);
        }

        for id in &full_output.textures_delta.free {
            self.renderer.free_texture(id);
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}
