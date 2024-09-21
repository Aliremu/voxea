use crate::app::App;
use crate::plugin;
use crate::window::{Render, WindowContext};
use cpal::traits::DeviceTrait;
use egui::vec2;
use log::info;
use voxea_alloc::perf;
use voxea_alloc::perf::PerfTrace;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::platform::windows::{WindowAttributesExtWindows, WindowExtWindows};
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::window::{Window as WinitWindow, WindowAttributes, WindowId};

pub fn init(cx: &mut App, event_loop: &ActiveEventLoop, parent: &WinitWindow) {
    let window_handle = parent.window_handle().unwrap().as_raw();

    let mut window_attributes = WindowAttributes::default()
        .with_title("Settings")
        .with_inner_size(PhysicalSize::new(800.0, 600.0))
        .with_visible(true);

    let hwnd = match window_handle {
        RawWindowHandle::Win32(handle) => handle.hwnd.get(),
        _ => todo!("Not running on Windows"),
    };

    // Sets the owner window to the parent, so it becomes a modal
    window_attributes = window_attributes
        .with_owner_window(hwnd)
        .with_clip_children(false);

    let hosts = voxea_audio::enumerate_hosts();

    let inputs = voxea_audio::enumerate_input_devices(hosts.get(0).unwrap())
        .iter()
        .filter_map(|d| d.name().ok())
        .collect();
    let outputs = voxea_audio::enumerate_output_devices(hosts.get(0).unwrap())
        .iter()
        .filter_map(|d| d.name().ok())
        .collect();

    let selected_host = hosts.get(0).unwrap().clone();

    let hosts = hosts
        .iter()
        .map(|h| h.name().to_string())
        .collect::<Vec<String>>();

    let selected_input = voxea_audio::default_input_device(&selected_host)
        .name()
        .unwrap();
    let selected_output = voxea_audio::default_output_device(&selected_host)
        .name()
        .unwrap();
    let selected_host = selected_host.name().to_string();

    let settings = Settings {
        hosts,
        selected_host,
        inputs,
        selected_input,
        outputs,
        selected_output,
        plugins_path: dirs::config_dir()
            .unwrap()
            .join("Voxea\\Plugins")
            .to_str()
            .unwrap()
            .to_string(),
        // plugins_path: "C:\\Users\\William\\AppData\\Roaming\\Voxea\\Plugins".to_string(),
        parent_window: Some(parent.id()),
    };

    let window_id = cx
        .open_window(
            event_loop,
            Some(window_attributes),
            Some(Box::new(settings)),
            true,
        )
        .unwrap();

    // cx.set_enable_all_other_windows(&window_id, false);
}

#[derive(Default)]
pub struct Settings {
    pub(crate) hosts: Vec<String>,
    pub(crate) selected_host: String,
    pub(crate) inputs: Vec<String>,
    pub(crate) selected_input: String,
    pub(crate) outputs: Vec<String>,
    pub(crate) selected_output: String,
    pub(crate) plugins_path: String,
    pub(crate) parent_window: Option<WindowId>,
}

impl Settings {
    fn select_host(&mut self, host: String) {
        info!("Selecting new host: {}", host);

        voxea_audio::enumerate_hosts()
            .iter()
            .find(|&h| h.name() == host)
            .inspect(|&id| {
                let inputs = voxea_audio::enumerate_input_devices(id)
                    .iter()
                    .filter_map(|d| d.name().ok())
                    .collect();
                let outputs = voxea_audio::enumerate_output_devices(id)
                    .iter()
                    .filter_map(|d| d.name().ok())
                    .collect();

                let selected_input = voxea_audio::default_input_device(id).name().unwrap();
                let selected_output = voxea_audio::default_output_device(id).name().unwrap();

                self.inputs = inputs;
                self.outputs = outputs;

                self.selected_input = selected_input;
                self.selected_output = selected_output;
            });
    }
}

impl Render for Settings {
    fn window_event(
        &mut self,
        cx: &mut WindowContext,
        event_loop: &ActiveEventLoop,
        event: &WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                // Re-enables the parent window
                cx.app
                    .get_window(&self.parent_window.unwrap())
                    .unwrap()
                    .window
                    .set_enable(true);
                // cx.app.set_enable_all_other_windows(&cx.window.window.id(), true);
            }
            _ => {}
        }
    }

    fn render(&mut self, cx: &mut WindowContext, event_loop: &ActiveEventLoop) {
        let window = &mut cx.window;
        let inner_size = window
            .window
            .inner_size()
            .to_logical::<f32>(window.window.scale_factor());

        let width = inner_size.width;
        let height = inner_size.height;

        perf::begin_perf!();

        window.ui2(|cx| {
            egui::CentralPanel::default().show(cx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let truncated = |s: &str, ui: &egui::Ui| {
                        let mut job = egui::text::LayoutJob::default();
                        let format = egui::TextFormat {
                            font_id: egui::TextStyle::Button.resolve(ui.style()),
                            ..Default::default()
                        };
                        job.append(s, 0.0, format);
                        job.wrap = egui::text::TextWrapping {
                            max_rows: 1,
                            break_anywhere: true,
                            ..Default::default()
                        };
                        job
                    };

                    ui.group(|ui| {
                        ui.heading("Audio Settings:");
                        ui.horizontal(|ui| {
                            ui.label("Audio Host");

                            let combo = egui::ComboBox::from_id_source("host")
                                .selected_text(truncated(&self.selected_host, ui))
                                .truncate()
                                .width(ui.available_width())
                                .show_ui(ui, |ui| {
                                    self.hosts
                                        .iter()
                                        .enumerate()
                                        .map(|(idx, name)| {
                                            ui.selectable_value(
                                                &mut self.selected_host,
                                                name.clone(),
                                                name,
                                            )
                                        })
                                        .reduce(|acc, r| acc | r)
                                        .unwrap()
                                });

                            let response = combo.inner.unwrap_or(combo.response);

                            if response.changed() {
                                self.select_host(self.selected_host.clone());
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label("Input Device");
                            egui::ComboBox::from_id_source("audio_input")
                                .selected_text(truncated(&self.selected_input, ui))
                                .truncate()
                                .width(ui.available_width())
                                .show_ui(ui, |ui| {
                                    for (idx, name) in self.inputs.iter().enumerate() {
                                        ui.selectable_value(
                                            &mut self.selected_input,
                                            name.clone(),
                                            name,
                                        );
                                    }
                                });
                        });

                        ui.horizontal(|ui| {
                            ui.label("Output Device");
                            egui::ComboBox::from_id_source("audio_output")
                                .selected_text(truncated(&self.selected_output, ui))
                                .truncate()
                                .width(ui.available_width())
                                .show_ui(ui, |ui| {
                                    for (idx, name) in self.outputs.iter().enumerate() {
                                        ui.selectable_value(
                                            &mut self.selected_output,
                                            name.clone(),
                                            name,
                                        );
                                    }
                                });
                        });
                    });

                    ui.group(|ui| {
                        ui.heading("Plugins:");
                        ui.horizontal(|ui| {
                            ui.label("Plugins Directory");
                            let file = egui::ComboBox::from_id_source("plugins_directory")
                                .selected_text(truncated(&self.plugins_path, ui))
                                .truncate()
                                .width(ui.available_width())
                                .show_ui(ui, |ui| {});

                            if file.response.clicked() {
                                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                    self.plugins_path = path.to_str().unwrap().to_string();
                                }
                            }
                        });
                        ui.label(format!("{} plugins loaded", plugin::get_plugins()));
                    });

                    // StripBuilder::new(ui)
                    //     .size(egui_extras::Size::exact(available_width))
                    //     .vertical(|mut strip| {
                    //         strip.cell(|ui| {
                    ui.group(|ui| {
                        ui.heading("Credits:");
                        ui.label("Author: William Zhang");
                        ui.label("Version: 0.0.1");
                        ui.label("Voxea");
                        ui.allocate_space(vec2(ui.available_width(), 0.0));
                    });

                    ui.group(|ui| {
                        ui.heading("Debug Info:");

                        for entry in perf::registry() {
                            let stats = entry.1;

                            ui.strong(*entry.0);

                            ui.label(format!(
                                "- Time Elapsed: {:.2} ms",
                                stats.timer.last_elapsed as f64 / 1000.0
                            ));
                            ui.label(format!(
                                "- Memory Allocated: {:.2} KB",
                                stats.memory.allocated as f64 / 1000.0
                            ));
                            ui.label(format!(
                                "- Memory Freed: {:.2} KB",
                                stats.memory.freed as f64 / 1000.0
                            ));
                            ui.label(format!(
                                "- Peak Memory Usage: {:.2} KB",
                                stats.memory.peak_usage as f64 / 1000.0
                            ));
                        }

                        let stats = perf::total_memory();
                        ui.label(format!("{}, {}", stats.0, stats.1));
                        ui.allocate_space(vec2(ui.available_width(), 0.0));
                    });
                    //     });
                    // });
                });
            });
        });
    }
}
