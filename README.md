# Voxea [WIP]
Digital audio processing application written in Rust with plugin support.

## Installation
```bash
cargo run --release
```

## Road Map
- [X] Satisfiable state + window management system
- [X] Integration with egui
- [X] Audio IO selection
- [ ] Comprehensive audio signal pipeline
- [ ] VST3 loading library

## Crates
- voxea: Front end for the app
- voxea_alloc: Custom global memory allocator wrapped around MiMalloc for tracking heap memory allocations and other performance information
- voxea_plugin: Plugin SDK for creating WASM plugins
- voxea_audio: Library for handling audio
- voxea_vst: VST3 library
- egui_winit: Forked from https://github.com/emilk/egui/tree/master/crates/egui-winit