#[cfg(target_arch = "wasm32")]
pub fn logger_init() {
    console_log::init_with_level(log::Level::Info).unwrap();
    //wasm_logger::init(wasm_logger::Config::default());
}

#[cfg(not(target_arch = "wasm32"))]
pub fn logger_init() {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );
}
