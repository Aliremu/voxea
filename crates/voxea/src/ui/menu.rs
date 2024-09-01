use cpal::traits::DeviceTrait;
use egui_dropdown::DropDownBox;
use log::info;
use winit::dpi::PhysicalSize;
use winit::event_loop::ActiveEventLoop;
use winit::platform::windows::WindowExtWindows;
use winit::window::WindowAttributes;
use crate::App;
use crate::ui::settings;
use crate::window::{Render, WindowContext};

pub fn init(cx: &mut App, event_loop: &ActiveEventLoop) {
    let window_attributes = WindowAttributes::default()
        .with_title("Voxea 0.1")
        .with_inner_size(PhysicalSize::new(1600, 1200));

    cx.open_window(event_loop, Some(window_attributes), Some(Box::new(Menu::default()))).expect("Failed to open menu");
}

#[derive(Default)]
pub struct Menu {

}

impl Render for Menu {
    fn render(&mut self, cx: &mut WindowContext, event_loop: &ActiveEventLoop) {
        let mut window = &mut cx.window;
        let inner_size = window.window.inner_size()
            .to_logical::<f32>(window.window.scale_factor());

        let width = inner_size.width;
        let height = inner_size.height;

        let app = &mut cx.app;

        window.cx.ui2(&window.window, |cx| {
            egui::TopBottomPanel::top("top_panel")
                .exact_height(32.0)
                .show(cx, |ui| {
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        let button = ui.button("Settings");

                        // ui.add(
                        //     // [ui.available_width() - 5.0, 32.0],
                        //     egui::TextEdit::singleline(&mut self.text)
                        //         .hint_text(egui::text::LayoutJob::simple_singleline(
                        //             "Search...".to_string(),
                        //             egui::FontId::default(),
                        //             egui::Color32::default(),
                        //         ))
                        //         .desired_width(f32::INFINITY)
                        //         .vertical_align(egui::Align::Center),
                        // );

                        // ui.add(
                        //     DropDownBox::from_iter(
                        //         &self.inputs,
                        //         "audio_inpts",
                        //         &mut self.input_search,
                        //         |ui, text| ui.selectable_label(false, text)
                        //     )
                        // );

                        if button.clicked() {
                            // info!("Clicked!");
                            // std::thread::spawn(|| {
                            //     voxea_audio::beep().unwrap();
                            // });

                            settings::init(*app, event_loop, &window.window);
                            window.window.set_enable(false);
                        }
                    });
                });
        });
    }
}