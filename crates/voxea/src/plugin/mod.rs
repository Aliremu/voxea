//use crate::renderer;
//use anyhow::{anyhow, Result};
//use log::{info, warn};
//use std::fs;
//use std::sync::OnceLock;
//use wasmtime::component::{Component, Linker, ResourceTable};
//use wasmtime::{Config, Engine, Store};
//use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiView};
//
//pub struct PluginContext {
//    pub(crate) plugins: Vec<Plugin>,
//    pub(crate) engine: Engine,
//    pub(crate) linker: Linker<MyState>,
//
//    pub(crate) store: Store<MyState>,
//    pub(crate) signal: Vec<f64>,
//}
//
//static mut CONTEXT: OnceLock<PluginContext> = OnceLock::new();
//
//pub fn init() -> Result<()> {
//    // Modules can be compiled through either the text or binary format
//    let mut config = Config::new();
//    config.async_support(false);
//    let engine = Engine::new(&config)?;
//
//    // let module = Module::new(&engine, wat)?;
//
//    // Create a `Linker` which will be later used to instantiate this module.
//    // Host functionality is defined by name within the `Linker`.
//    let mut linker = Linker::<MyState>::new(&engine);
//    linker
//        .instance("sdk:component/logger")?
//        .func_wrap("log", |_, param: (String,)| {
//            println!("{}", param.0);
//            Ok(())
//        })?;
//
//    let mut registry = linker.instance("sdk:component/registry")?;
//
//    registry.func_wrap::<_, _, (f64,)>("get-signal", |_, param: (u64,)| unsafe {
//        let signal = CONTEXT
//            .get()
//            .expect("Plugin Context not initialized!")
//            .signal
//            .get(param.0 as usize)
//            .unwrap();
//        Ok((*signal,))
//    })?;
//
//    registry.func_wrap::<_, _, (f64,)>("set-signal", |_, param: (u64, f64)| unsafe {
//        let signal = CONTEXT
//            .get_mut()
//            .expect("Plugin Context not initialized!")
//            .signal
//            .get(param.0 as usize)
//            .unwrap();
//        Ok((*signal,))
//    })?;
//
//    // All wasm objects operate within the context of a "store". Each
//    // `Store` has a type parameter to store host-specific data, which in
//    // this case we're using `4` for.
//    let wasi = WasiCtxBuilder::new().inherit_stdio().inherit_args().build();
//    let store = Store::new(
//        &engine,
//        MyState {
//            ctx: wasi,
//            table: ResourceTable::new(),
//        },
//    );
//
//    wasmtime_wasi::add_to_linker_sync(&mut linker)?;
//
//    let cx = PluginContext {
//        plugins: Vec::new(),
//        engine,
//        linker,
//        store,
//        signal: vec![32.0, 24.0, 16.0],
//    };
//
//    unsafe {
//        CONTEXT.get_or_init(|| cx);
//        Ok(())
//    }
//}
//
//pub fn load_plugins() -> Result<()> {
//    let Some(cx) = (unsafe { CONTEXT.get_mut() }) else {
//        warn!("Could not load plugins! Plugin Context not initialized!");
//        return Err(anyhow!("Could not load plugins!"));
//    };
//
//    let paths = fs::read_dir("./plugins").expect("Could not read plugins directory!");
//
//    let plugins = paths
//        .into_iter()
//        .filter_map(|p| p.ok())
//        .map(|p| p.path())
//        .filter(|path| path.extension().map_or(false, |ext| ext == "wasm"));
//
//    for plugin in plugins {
//        info!("Loading {}!", plugin.to_str().unwrap());
//        let component = Component::from_file(&cx.engine, plugin.to_str().unwrap())?;
//        let instance = Plugin::instantiate(&mut cx.store, &component, &cx.linker)?;
//
//        let icon = instance
//            .sdk_component_plugin_api()
//            .call_icon(&mut cx.store)?;
//
//        renderer::get_mut().create_texture_from_memory(&icon);
//
//        info!("Enabling {}!", plugin.to_str().unwrap());
//        let result = instance
//            .sdk_component_plugin_api()
//            .call_enable(&mut cx.store)?;
//
//        println!("{:?}", result);
//
//        unsafe {
//            cx.plugins.push(instance);
//        }
//    }
//
//    Ok(())
//}
//
//pub fn process_signal() {
//    unsafe {
//        let Some(cx) = CONTEXT.get_mut() else {
//            warn!("Plugin Context not initialized!");
//            return;
//        };
//
//        let signal = [32.0, 24.0, 16.0];
//        let signal = signal.as_ptr() as u64;
//
//        for plugin in &cx.plugins {
//            // plugin
//            //     .sdk_component_plugin_api()
//            //     .call_process_signal(&mut cx.store, signal)
//            //     .unwrap();
//        }
//    }
//}
//
//pub fn get_plugins() -> usize {
//    unsafe { CONTEXT.get().unwrap().plugins.len() }
//}
//
//struct MyState {
//    ctx: WasiCtx,
//    table: ResourceTable,
//}
//
//impl WasiView for MyState {
//    fn table(&mut self) -> &mut ResourceTable {
//        &mut self.table
//    }
//    fn ctx(&mut self) -> &mut WasiCtx {
//        &mut self.ctx
//    }
//}
//
//wasmtime::component::bindgen!({
//    path: "../voxea_plugin/wit/world.wit",
//    world: "plugin",
//    async: false
//});
