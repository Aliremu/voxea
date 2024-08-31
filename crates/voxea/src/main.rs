use winit::application::ApplicationHandler;
use winit::event::{StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window as WinitWindow, WindowAttributes, WindowId};

use egui::{Pos2, ViewportBuilder};
use egui_wgpu::wgpu;
use log::{info, warn};
use std::time::{Duration, Instant};
use std::{borrow::Cow, sync::Arc};
use tracing_subscriber::fmt::format;
use tracing_subscriber::fmt::format::Format;
use tracing_subscriber::fmt::time::LocalTime;
use winit::{dpi::PhysicalSize, window::Window as NativeWindow};

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
}

impl RenderContext {
    pub fn new(egui_ctx: egui::Context, window: Arc<NativeWindow>) -> Self {
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

        let instance = wgpu::Instance::default();
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

        let config = surface.get_default_config(&adapter, width, height).unwrap();
        surface.configure(&device, &config);

        let renderer =
            egui_wgpu::Renderer::new(&device, wgpu::TextureFormat::Bgra8UnormSrgb, None, 1);

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
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
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

        let command_buffers = self.renderer.update_buffers(
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

            rpass.set_pipeline(&self.render_pipeline);
            rpass.draw(0..3, 0..1);
        }

        for id in &full_output.textures_delta.free {
            self.renderer.free_texture(id);
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}

pub struct Window {
    pub(crate) window: Arc<WinitWindow>,
    pub(crate) cx: RenderContext,
}

impl Window {
    pub fn new(event_loop: &ActiveEventLoop) -> Self {
        let egui_ctx = egui::Context::default();
        egui_ctx.set_fonts(egui::FontDefinitions::default());
        egui_ctx.set_style(egui::Style::default());

        let window = Arc::new(
            egui_winit::create_window(&egui_ctx, event_loop, &ViewportBuilder::default())
                .expect("Failed to create window!"),
        );

        let cx = RenderContext::new(egui_ctx, window.clone());

        Self { window, cx }
    }

    #[inline]
    pub fn on_window_event(&mut self, event: &WindowEvent) -> egui_winit::EventResponse {
        self.cx.egui_state.on_window_event(&self.window, event)
    }

    #[inline]
    pub fn resize(&mut self, physical_size: PhysicalSize<u32>) {
        self.cx.resize(physical_size);
    }
    #[inline]
    pub fn render(&mut self) {
        self.cx.render(&self.window);
    }
}

#[derive(Default)]
pub struct App {
    window: Option<Window>,
    wait_cancelled: bool,
}

const WAIT_TIME: Duration = Duration::from_micros(16666);

impl ApplicationHandler for App {
    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        self.wait_cancelled = matches!(cause, StartCause::WaitCancelled { .. });
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        voxea_audio::enumerate_hosts().unwrap();

        self.window = Some(Window::new(event_loop));
    }
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let window = self.window.as_mut().unwrap();

        let response = window.on_window_event(&event);

        match event {
            WindowEvent::Resized(size) => {
                info!("Resizing window to {size:?}");
                window.resize(size);
            }

            WindowEvent::RedrawRequested => {
                if response.repaint {
                    window.cx.ui(|cx| {
                        egui::CentralPanel::default().show(cx, |ui| {
                            let button = ui.button("Click me!");

                            if button.clicked() {
                                info!("Clicked!");
                                std::thread::spawn(|| {
                                    voxea_audio::beep().unwrap();
                                });
                            }
                        });
                    });
                }

                window.render();
            }

            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if !self.wait_cancelled {
            self.window.as_ref().unwrap().window.request_redraw();

            event_loop.set_control_flow(ControlFlow::WaitUntil(Instant::now() + WAIT_TIME));
        }
    }
}

fn main() {
    tracing_subscriber::fmt()
        .with_timer(LocalTime::rfc_3339())
        .init();

    let event_loop = EventLoop::builder()
        .build()
        .expect("Could not create event loop!");

    let mut app = App::default();
    let _ = event_loop.run_app(&mut app);
}
