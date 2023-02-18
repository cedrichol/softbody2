cargo build --target wasm32-unknown-unknown
wasm-bindgen ./target/wasm32-unknown-unknown/debug/softbody_wasm.wasm --out-dir bindgen --target web