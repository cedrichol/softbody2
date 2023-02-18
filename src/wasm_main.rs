#![cfg(target_arch = "wasm32")]
#![allow(special_module_name)]
#[allow(warnings)]
mod main;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    main::main();
    Ok(())
}
