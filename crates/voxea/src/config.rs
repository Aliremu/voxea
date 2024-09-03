use std::sync::OnceLock;

static mut REGISTRY: OnceLock<Config<_>> = OnceLock::new();

pub trait ConfigType {}

pub struct Config<T: ConfigType> {}

pub fn get<T: ConfigType>() -> Config<T> {
    unsafe {}
}
