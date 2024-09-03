use crate::ui::settings;
use crate::window::{Render, WindowContext};
use crate::{plugin, App};
use cpal::traits::DeviceTrait;
use egui::{pos2, Color32};
use egui_dropdown::DropDownBox;
use log::info;
use winit::dpi::PhysicalSize;
use winit::event_loop::ActiveEventLoop;
use winit::platform::windows::WindowExtWindows;
use winit::window::WindowAttributes;

pub fn init(cx: &mut App, event_loop: &ActiveEventLoop) {
    let window_attributes = WindowAttributes::default()
        .with_title("Voxea 0.1")
        .with_inner_size(PhysicalSize::new(1600, 900));

    cx.open_window(
        event_loop,
        Some(window_attributes),
        Some(Box::new(Menu::default())),
    )
    .expect("Failed to open menu");
}

#[derive(Default)]
pub struct Menu {}

impl Render for Menu {
    fn render(&mut self, cx: &mut WindowContext, event_loop: &ActiveEventLoop) {
        plugin::process_signal();

        let mut window = &mut cx.window;

        let app = &mut cx.app;

        window.cx.draw_triangle();
        let texture_id = window.cx.fbo_id;

        window.cx.ui2(&window.window, |cx| {
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
                            settings::init(*app, event_loop, &window.window);

                            // Disables the parent window
                            window.window.set_enable(false);
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
                ui.painter().image(texture_id, rect, uv, Color32::WHITE);
            });
        });
    }
}
