use anyhow::Result;
use std::sync::OnceLock;
use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiView};

static mut REGISTRY: OnceLock<Vec<Plugin>> = OnceLock::new();
static mut STORE: OnceLock<Store<MyState>> = OnceLock::new();
static mut SIGNAL: OnceLock<Vec<f64>> = OnceLock::new();

pub fn run_plugins() -> Result<()> {
    unsafe {
        REGISTRY.get_or_init(|| Vec::new());
        SIGNAL.get_or_init(|| vec![32.0, 24.0, 16.0]);
    }

    // Modules can be compiled through either the text or binary format
    let mut config = Config::new();
    config.async_support(false);
    let engine = Engine::new(&config)?;

    // let module = Module::new(&engine, wat)?;

    // Create a `Linker` which will be later used to instantiate this module.
    // Host functionality is defined by name within the `Linker`.
    let mut linker = Linker::<MyState>::new(&engine);
    linker
        .instance("sdk:component/logger")?
        .func_wrap("log", |_, param: (String,)| {
            println!("{}", param.0);
            Ok(())
        })?;

    let mut registry = linker.instance("sdk:component/registry")?;

    registry.func_wrap::<_, _, (bool,)>("add-entity", |_, param: (String,)| {
        println!("Adding entity {}", param.0);
        Ok((false,))
    })?;

    registry.func_wrap::<_, _, (f64,)>("get-signal", |_, param: (u64,)| unsafe {
        let signal = SIGNAL
            .get_or_init(|| Vec::new())
            .get(param.0 as usize)
            .unwrap();
        Ok((*signal,))
    })?;

    // All wasm objects operate within the context of a "store". Each
    // `Store` has a type parameter to store host-specific data, which in
    // this case we're using `4` for.
    let wasi = WasiCtxBuilder::new().inherit_stdio().inherit_args().build();
    let mut store = Store::new(
        &engine,
        MyState {
            ctx: wasi,
            table: ResourceTable::new(),
        },
    );

    wasmtime_wasi::add_to_linker_sync(&mut linker)?;

    let plugins = ["test_plugin"];
    for plugin in plugins {
        let component = Component::from_file(&engine, "plugins/".to_owned() + plugin + ".wasm")?;
        let instance = Plugin::instantiate(&mut store, &component, &linker)?;
        let result = instance
            .sdk_component_plugin_api()
            .call_enable(&mut store)?;

        println!("{:?}", result);

        unsafe {
            REGISTRY.get_mut_or_init(|| Vec::new()).push(instance);
        }
    }

    unsafe {
        STORE.get_or_init(|| store);
    }

    Ok(())
}

pub fn process_signal() {
    unsafe {
        let signal = [32.0, 24.0, 16.0];
        let signal = signal.as_ptr() as u64;

        for plugin in REGISTRY.get_or_init(|| Vec::new()) {
            plugin
                .sdk_component_plugin_api()
                .call_process_signal(STORE.get_mut().unwrap(), signal)
                .unwrap();
        }
    }
}

struct MyState {
    ctx: WasiCtx,
    table: ResourceTable,
}

impl WasiView for MyState {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.ctx
    }
}

wasmtime::component::bindgen!({
    path: "../voxea_plugin/wit/world.wit",
    world: "plugin",
    async: false
});
