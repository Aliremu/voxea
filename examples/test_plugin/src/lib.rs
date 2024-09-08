use std::collections::HashMap;
use voxea_plugin::{logger, registry, VoxeaPlugin};
struct MyPlugin;

impl VoxeaPlugin for MyPlugin {
    fn icon() -> Vec<u8> {
        let file = include_bytes!("../assets/32.png");
        file.to_vec()
    }

    fn enable() -> i32 {
        let mut map = HashMap::new();

        map.insert(123, "hello");
        logger::log(map[&123]);

        2343
    }

    fn disable() -> i32 {
        123
    }

    fn process_signal(ptr: u64) {
        unsafe {
            let signal = registry::get_signal(0);
            let log = format!("Process Signal at {:?} is: {}", ptr as *const u64, signal);

            logger::log(&log);
        }
    }
}

voxea_plugin::export!(MyPlugin with_types_in voxea_plugin::bindings);
