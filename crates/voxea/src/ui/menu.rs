use crate::ui::{plugview, settings};
use crate::window::{Render, WindowContext};
use crate::{renderer, App};
use egui::load::SizedTexture;
use egui::pos2;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::platform::windows::WindowExtWindows;
use winit::window::WindowAttributes;

pub fn init(cx: &mut App, event_loop: &ActiveEventLoop) {
    let window_attributes = WindowAttributes::default()
        .with_title("Voxea 0.1")
        .with_inner_size(PhysicalSize::new(1600, 900));

    let window = cx
        .open_window(
            event_loop,
            Some(window_attributes),
            Some(Box::new(Menu::default())),
            true,
        )
        .expect("Failed to open menu");
}

#[derive(Default)]
pub struct Menu {}

impl Render for Menu {
    fn window_event(
        &mut self,
        cx: &mut WindowContext,
        event_loop: &ActiveEventLoop,
        event: &WindowEvent,
    ) {
        match event {
            WindowEvent::KeyboardInput {
                event,
                is_synthetic,
                ..
            } => {
                if !is_synthetic && !event.repeat && event.state == ElementState::Pressed {
                    if event.physical_key == KeyCode::KeyJ {
                        plugview::init(cx.app, event_loop, 1);
                    }

                    if event.physical_key == KeyCode::KeyK {
                        plugview::init(cx.app, event_loop, 2);
                    }

                    if event.physical_key == KeyCode::KeyL {
                        plugview::init(cx.app, event_loop, 3);
                    }
                }
            }

            _ => {}
        }
    }
    fn render(&mut self, cx: &mut WindowContext, event_loop: &ActiveEventLoop) {
        let window = &mut cx.window;

        let app = &mut cx.app;

        let texture_id = window.backend.as_ref().unwrap().fbo_id;

        let parent = window.window.clone();

        window.ui2(|cx| {
            egui::TopBottomPanel::top("top_panel")
                .exact_height(40.0)
                .frame(egui::Frame::none().inner_margin(4.0))
                .show(cx, |ui| {
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        ui.visuals_mut().button_frame = false;

                        let _ = ui.button("File");
                        let button = ui.button("Settings");
                        let help = ui.button("Help");

                        if button.clicked() {
                            settings::init(app, event_loop, &parent);

                            // Disables the parent window
                            parent.set_enable(false);
                        }
                    });
                });

            egui::CentralPanel::default().show(cx, |ui| {
                let rect = egui::Rect {
                    min: ui.cursor().left_top(),
                    max: (ui.cursor().left_top() + ui.available_size()),
                };
                let uv = egui::Rect {
                    min: pos2(0.0, 0.0),
                    max: pos2(1.0, 1.0),
                };

                let rend = renderer::get();

                if rend.textures.len() > 0 {
                    let img =
                        SizedTexture::new(rend.textures.get(0).unwrap().2, ui.available_size());

                    // info!("{:?}", rend.textures.get(0).unwrap());

                    ui.add(egui::Image::new(img));
                    // ui.painter().image(rend.textures.get(0).unwrap().2, rect, uv, Color32::from_rgba_premultiplied(255, 255, 255, 1));
                }
            });
        });
    }
}
