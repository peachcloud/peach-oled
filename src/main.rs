use std::process;

use log::error;

fn main() {
    // initialize the logger
    env_logger::init();

    // handle errors returned from `run`
    if let Err(e) = peach_oled::run() {
        error!("Application error: {}", e);
        process::exit(1);
    }
}
