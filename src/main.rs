/**
 * Copyright (c) 2018, Sébastien Blin <sebastien.blin@enconn.fr>
 * All rights reserved.
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are met:
 *
 * * Redistributions of source code must retain the above copyright
 *  notice, this list of conditions and the following disclaimer.
 * * Redistributions in binary form must reproduce the above copyright
 *  notice, this list of conditions and the following disclaimer in the
 *  documentation and/or other materials provided with the distribution.
 * * Neither the name of the University of California, Berkeley nor the
 *  names of its contributors may be used to endorse or promote products
 *  derived from this software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE REGENTS AND CONTRIBUTORS ``AS IS'' AND ANY
 * EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
 * WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
 * DISCLAIMED. IN NO EVENT SHALL THE REGENTS AND CONTRIBUTORS BE LIABLE FOR ANY
 * DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
 * (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
 * LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND
 * ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
 * SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 **/

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
    // TODO if not config, UI to create it
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
    let mut api = API::new(shared_manager,
                           String::from(config["api_listener"].as_str().unwrap_or("")));
    api.start();
    let _ = test.join();
    // TODO proper quit
    // TODO ncurses for UI
}
