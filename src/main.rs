use std::process::exit;

extern crate serde;

pub mod config;
pub mod executor;

fn main() {

    let manager = config::ConfigManager::start().unwrap();
    let mut runtime = executor::Runtime::create(manager);

    match runtime.start() {
        Ok(_) => {
            exit(0)
        },
        Err(err) => {
            println!("{}", err);
            exit(1);
        },
    }
}
