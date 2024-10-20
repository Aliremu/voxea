use std::collections::HashMap;
use voxea_plugin::{logger, registry, VoxeaPlugin};
struct MyPlugin;

impl VoxeaPlugin for MyPlugin {
    fn enable() -> i32 {
        let mut map = HashMap::new();

        map.insert(123, "world");
        logger::log(map[&123]);

        9999
    }

    fn disable() -> i32 {
        999
    }

    fn process_signal(ptr: u64) {
        unsafe {
            let signal = registry::set_signal(0, 0.0);
            let log = format!("Process Signal 2 at {:?} is: {}", ptr as *const u64, signal);

            logger::log(&log);
        }
    }
}
voxea_plugin::export!(MyPlugin with_types_in voxea_plugin::bindings);
