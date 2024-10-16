use crate::ui::menu;
use crate::{renderer, App};
use anyhow::Result;
use egui::ViewportBuilder;
use egui_winit::{apply_viewport_builder_to_window, create_winit_window_attributes, EventResponse};
use log::{info, warn};
use std::sync::Arc;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::window::WindowAttributes;
use winit::{dpi::PhysicalSize, window::Window as WinitWindow};

pub struct WindowContext<'a> {
    pub(crate) app: &'a mut App,
    pub(crate) window: &'a mut Window,
}

pub trait Render: 'static {
    fn window_event(
        &mut self,
        _cx: &mut WindowContext,
        _event_loop: &ActiveEventLoop,
        _event: &WindowEvent,
    ) {
    }

    fn on_open(&mut self, _cx: &mut WindowContext) {}

    fn render(&mut self, cx: &mut WindowContext, event_loop: &ActiveEventLoop);
}

pub struct Backend {
    pub(crate) surface: wgpu::Surface<'static>,
    pub(crate) config: wgpu::SurfaceConfiguration,
    pub(crate) render_pipeline: wgpu::RenderPipeline,
    pub(crate) fbo: wgpu::Texture,
    pub(crate) fbo_view: wgpu::TextureView,
    pub(crate) fbo_id: egui::TextureId,

    pub(crate) ui_cmds: Vec<Box<dyn Fn(&egui::Context) + 'static>>,
    pub(crate) egui_state: egui_winit::State,
}

/// A wrapper around the winit window
pub struct Window {
    pub(crate) window: Arc<WinitWindow>,
    pub(crate) view: Option<Box<dyn Render + 'static>>,
    pub(crate) backend: Option<Backend>,

    pub(crate) running: bool,

    // winit bug where random resize events are sent on startup. https://github.com/rust-windowing/winit/issues/2094
    pub(crate) init: bool,
}

impl Window {
    /// Creates a window and initializes the [`RenderContext`] and egui states
    /// Must be called in the main thread to have access to [`ActiveEventLoop`]
    pub fn new(
        event_loop: &ActiveEventLoop,
        window_attributes: Option<WindowAttributes>,
        view: Option<Box<dyn Render + 'static>>,
        backend: bool,
    ) -> Result<Self> {
        if !backend {
            let window = Arc::new(
                event_loop
                    .create_window(window_attributes.map_or(WindowAttributes::default(), |w| w))?,
            );

            return Ok(Self {
                window,
                view,
                backend: None,
                running: true,
                init: false,
            });
        }

        let render_context = renderer::get_mut();

        let egui_ctx = egui::Context::default();
        egui_ctx.set_fonts(egui::FontDefinitions::default());
        egui_ctx.set_style(egui::Style::default());

        let viewport_builder = ViewportBuilder::default();

        let window_attributes = window_attributes.map_or(
            create_winit_window_attributes(&egui_ctx, event_loop, viewport_builder.clone()),
            |v| v,
        );

        let window = Arc::new(event_loop.create_window(window_attributes)?);
        apply_viewport_builder_to_window(&egui_ctx, &window, &viewport_builder);

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

        let surface = render_context
            .create_surface(window.clone())
            .expect("Could not create surface for window!");

        // let swapchain_capabilities = surface.get_capabilities(&render_context.adapter);
        // let swapchain_format = swapchain_capabilities.formats[0];

        let pipeline_layout =
            render_context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[],
                    push_constant_ranges: &[],
                });

        let target_state = wgpu::ColorTargetState {
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            blend: Some(wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::Zero,
                    operation: wgpu::BlendOperation::Add,
                },
            }),
            write_mask: wgpu::ColorWrites::ALL,
        };

        let render_pipeline =
            render_context
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: None,
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &render_context.shader,
                        entry_point: "vs_main",
                        buffers: &[],
                        compilation_options: Default::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &render_context.shader,
                        entry_point: "fs_main",
                        compilation_options: Default::default(),
                        targets: &[Some(target_state)],
                    }),
                    primitive: wgpu::PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    multiview: None,
                });

        let config = surface
            .get_default_config(&render_context.adapter, width, height)
            .unwrap();
        // config.alpha_mode = wgpu::CompositeAlphaMode::PreMultiplied;
        info!("{:?}", config);

        surface.configure(&render_context.device, &config);

        let fbo = render_context
            .device
            .create_texture(&wgpu::TextureDescriptor {
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
        let fbo_id = render_context.renderer.register_native_texture(
            &render_context.device,
            &fbo_view,
            wgpu::FilterMode::Linear,
        );

        let backend = Backend {
            surface,
            config,
            render_pipeline,
            fbo,
            fbo_view,
            fbo_id,

            ui_cmds: Vec::new(),
            egui_state,
        };

        Ok(Self {
            window,
            view,

            backend: Some(backend),

            running: true,
            init: false,
        })
    }

    #[inline]
    pub fn on_window_event(
        &mut self,
        cx: &mut App,
        event_loop: &ActiveEventLoop,
        event: &WindowEvent,
    ) -> egui_winit::EventResponse {
        let repaint = self.backend.as_mut().map_or(true, |backend| {
            backend
                .egui_state
                .on_window_event(&self.window, event)
                .repaint
        });

        if let Some(mut view) = self.view.take() {
            let mut cx = WindowContext {
                app: cx,
                window: self,
            };

            view.window_event(&mut cx, event_loop, &event);

            self.view = Some(view);
        }

        match event {
            WindowEvent::CloseRequested => {
                self.running = false;
            }

            WindowEvent::Resized(size) => {
                if self.init {
                    info!("Resizing window to {size:?}");
                    self.resize(size);
                } else {
                    self.init = true;
                }
            }

            WindowEvent::KeyboardInput {
                event,
                is_synthetic,
                ..
            } => {
                if !is_synthetic
                    && !event.repeat
                    && event.state == ElementState::Pressed
                    && event.physical_key == KeyCode::KeyG
                {
                    menu::init(cx, event_loop);
                }
            }

            WindowEvent::RedrawRequested => {
                // Redraw the UI if egui deems it necessary, eg. state has been updated
                if repaint {
                    if let Some(mut view) = self.view.take() {
                        let mut cx = WindowContext {
                            app: cx,
                            window: self,
                        };

                        view.render(&mut cx, event_loop);

                        self.view = Some(view);
                    }
                }
            }
            _ => {}
        }

        EventResponse::default()
    }

    pub fn resize(&mut self, new_size: &PhysicalSize<u32>) {
        let Some(backend) = self.backend.as_mut() else {
            return;
        };

        if new_size.width > 0 && new_size.height > 0 {
            let cx = renderer::get();
            backend.config.width = new_size.width;
            backend.config.height = new_size.height;
            backend.surface.configure(&cx.device, &backend.config);
        }
    }

    pub fn ui2<F>(&mut self, mut f: F)
    where
        F: FnMut(&egui::Context),
    {
        let Some(backend) = self.backend.as_mut() else {
            return;
        };

        let render_context = renderer::get_mut();

        let input = backend.egui_state.take_egui_input(&self.window);
        let cx = backend.egui_state.egui_ctx();
        let full_output = backend.egui_state.egui_ctx().run(input, |cx| {
            f(cx);
        });

        backend.ui_cmds.clear();

        let clipped_primitives =
            cx.tessellate(full_output.shapes.clone(), full_output.pixels_per_point);

        for (id, image_delta) in &full_output.textures_delta.set {
            render_context.renderer.update_texture(
                &render_context.device,
                &render_context.queue,
                *id,
                image_delta,
            );
        }

        let desc = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [backend.config.width, backend.config.height],
            pixels_per_point: cx.pixels_per_point(),
        };

        let Ok(frame) = backend.surface.get_current_texture() else {
            // warn!("Failed to acquire next swap chain texture");
            return;
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = render_context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let _command_buffers = render_context.renderer.update_buffers(
            &render_context.device,
            &render_context.queue,
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

            render_context
                .renderer
                .render(&mut rpass, &clipped_primitives, &desc);
        }

        for id in &full_output.textures_delta.free {
            render_context.renderer.free_texture(id);
        }

        render_context.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    #[inline]
    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        warn!("Dropping window! {:?}", self.window.id());
    }
}
