#[path = "logging.rs"]
mod logging;

#[path = "softbody.rs"]
mod softbody;

#[allow(dead_code)]
pub fn main() {
    logging::logger_init();
    softbody::demo();
}
