export COMPONENT_ADAPTER_REACTOR=C:/WASM/wasi_snapshot_preview1.reactor.wasm
tinygo build -o go_plugin.wasm -target=wasi main.go
wasm-tools component embed --world plugin ./world.wit go_plugin.wasm -o go_plugin.embed.wasm
wasm-tools component new -o go_plugin.wasm --adapt wasi_snapshot_preview1="$COMPONENT_ADAPTER_REACTOR" go_plugin.embed.wasm