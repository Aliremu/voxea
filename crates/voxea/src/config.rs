// use std::path::Path;
// use std::sync::OnceLock;
//
// static mut REGISTRY: OnceLock<Config<dyn ConfigType + 'static>> = OnceLock::new();
//
// pub trait ConfigType {
//     fn file_path(&mut self) -> Path;
// }
//
// pub struct PluginConfig {
//     pub(crate) file_path: String
// }
//
// impl ConfigType for PluginConfig {
//     fn file_path(&mut self) -> String {
//         self.file_path.clone()
//     }
// }
//
// pub struct Config<T: ConfigType> {}
//
// pub fn get<T: ConfigType>() -> Config<T> {
//     unsafe {
//         REGISTRY.get_or_init()
//     }
// }
