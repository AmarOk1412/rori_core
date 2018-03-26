extern crate dbus;
extern crate env_logger;
extern crate iron;
#[macro_use]
extern crate log;
extern crate router;
extern crate rusqlite;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate time;

pub mod rori;

use rori::manager::Manager;
use rori::api::API;
use serde_json::{Value, from_str};
use std::io::prelude::*;
use std::sync::{Arc, Mutex};
use std::fs::File;
use std::thread;

#[allow(dead_code)]
fn main() {
    // Init logging
    env_logger::init();


    // This script load config from config.json
    let mut file = File::open("config.json").ok()
        .expect("Config file not found");
    let mut config = String::new();
    file.read_to_string(&mut config).ok()
        .expect("failed to read!");
    let config: Value = from_str(&*config).ok()
                        .expect("Incorrect config file. Please check config.json");

    let shared_manager : Arc<Mutex<Manager>> = Arc::new(Mutex::new(
        Manager::init(config["ring_id"].as_str().unwrap_or(""))
        .ok().expect("Can't initialize ConfigurationManager"))
    );
    let shared_manager_cloned = shared_manager.clone();
    let test = thread::spawn(move || {
        Manager::handle_signals(shared_manager_cloned);
    });
    let mut api = API::new(shared_manager);
    api.start();
    let _ = test.join();
    // TODO proper quit
    // TODO ncurses for UI
    // TODO comments
    // TODO README
    // TODO License
}
