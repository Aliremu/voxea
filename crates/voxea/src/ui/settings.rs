use cpal::traits::DeviceTrait;
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::platform::windows::{WindowAttributesExtWindows, WindowExtWindows};
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::window::{Window as WinitWindow, WindowAttributes, WindowId};
use crate::app::App;
use crate::window::{Render, WindowContext};

pub fn init(cx: &mut App, event_loop: &ActiveEventLoop, parent: &WinitWindow) {
    let window_handle = parent.window_handle().unwrap().as_raw();

    let mut window_attributes = WindowAttributes::default()
        .with_title("Settings")
        .with_inner_size(PhysicalSize::new(800.0f32, 600.0f32))
        .with_visible(true);

    let hwnd = match window_handle {
        RawWindowHandle::Win32(handle) => handle.hwnd.get(),
        _ => panic!("not running on Windows")
    };

    // `with_parent_window` is unsafe. Parent window must be a valid window.
    window_attributes = unsafe {
        window_attributes.with_owner_window(hwnd).with_clip_children(false)
    };

    let hosts = voxea_audio::enumerate_hosts();

    let inputs = voxea_audio::enumerate_input_devices(hosts.get(0).unwrap())
        .iter()
        .filter_map(|d| d.name().ok())
        .collect();
    let outputs = voxea_audio::enumerate_output_devices(hosts.get(0).unwrap())
        .iter()
        .filter_map(|d| d.name().ok())
        .collect();

    let settings = Settings {
        inputs,
        selected_input: String::new(),
        outputs,
        selected_output: String::new(),
        parent_window: Some(parent.id())
    };

    cx.open_window(event_loop, Some(window_attributes), Some(Box::new(settings))).unwrap();
}

#[derive(Default)]
pub struct Settings {
    pub(crate) inputs: Vec<String>,
    pub(crate) selected_input: String,
    pub(crate) outputs: Vec<String>,
    pub(crate) selected_output: String,
    pub(crate) parent_window: Option<WindowId>
}

impl Render for Settings {
    fn window_event(&mut self, cx: &mut WindowContext, event_loop: &ActiveEventLoop, event: &WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                cx.app.get_window(&self.parent_window.unwrap()).unwrap().window.set_enable(true);
            }
            _ => {}
        }
    }

    fn render(&mut self, cx: &mut WindowContext, event_loop: &ActiveEventLoop) {
        let mut window = &mut cx.window;
        let inner_size = window.window.inner_size()
            .to_logical::<f32>(window.window.scale_factor());

        let width = inner_size.width;
        let height = inner_size.height;

        window.cx.ui2(&window.window, |cx| {
            egui::CentralPanel::default()
                .show(cx, |ui| {
                    egui::ComboBox::from_label("Audio Input")
                        .selected_text(format!("{}", self.selected_input))
                        .show_ui(ui, |ui| {
                            for (idx, name) in self.inputs.iter().enumerate() {
                                ui.selectable_value(&mut self.selected_input, name.clone(), name);
                            }
                        });

                    egui::ComboBox::from_label("Audio Output")
                        .selected_text(format!("{}", self.selected_output))
                        .show_ui(ui, |ui| {
                            for (idx, name) in self.outputs.iter().enumerate() {
                                ui.selectable_value(&mut self.selected_output, name.clone(), name);
                            }
                        });
                });
        });
    }
}