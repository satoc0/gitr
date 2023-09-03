use std::process::exit;

extern crate serde;

pub mod config;
pub mod executor;

fn main() {
    match config::ConfigManager::start() {
        Ok(manager) => {
            let mut runtime: executor::Runtime = executor::Runtime::create(manager);

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
        Err(err) => { println!("{}", err) }
    }

    
}
