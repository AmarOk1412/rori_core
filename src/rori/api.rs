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
 
use iron::prelude::*;
use iron::Handler;
use iron::mime::Mime;
use iron::status;
use router::Router;
use rori::manager::Manager;
use serde_json;
use std::sync::{Arc, Mutex};

pub struct API {
    address: String,
    manager: Arc<Mutex<Manager>>
}

struct NameHandler {
    manager: Arc<Mutex<Manager>>
}

#[derive(Serialize, Deserialize)]
struct NameResponse {
    name: String,
    addr: String,
}

#[derive(Serialize, Deserialize)]
struct NameError {
    error: String,
}

impl Handler for NameHandler {
    fn handle(&self, request: &mut Request) -> IronResult<Response> {
        let content_type = "application/json".parse::<Mime>().unwrap();
        let name = request.extensions.get::<Router>().unwrap().find("name").unwrap_or("");
        info!("GET /name/{}", name);
        let ring_id = self.manager.lock().unwrap().server.get_ring_id(&String::from(name));
        // Some data structure.
        if ring_id.len() > 0 {
            let answer = NameResponse {
                name: String::from(name),
                addr: ring_id.replace("ring:", "0x"),
            };
            let response = serde_json::to_string(&answer).unwrap_or(String::new());
            return Ok(Response::with((content_type, status::Ok, response)))
        }
        let answer = NameError {
            error: String::from("name not registred")
        };
        let response = serde_json::to_string(&answer).unwrap_or(String::new());
        Ok(Response::with((content_type, status::NotFound, response)))
    }
}

impl API {
    pub fn new(manager: Arc<Mutex<Manager>>) -> API {
        API {
            address: String::from("0.0.0.0:1412"), // TODO change address
            manager: manager
        }
    }

    pub fn start(&mut self) {
        let mut router = Router::new();
        let name_handler = NameHandler {
            manager: self.manager.clone()
        };
        router.get("/name/:name", name_handler, "name");
        router.get("/help", API::help, "help");
        info!("start API endpoint at {}", self.address);
        Iron::new(router).http(&*self.address).unwrap();
    }

    pub fn help(_: &mut Request) -> IronResult<Response> {
        info!("GET /help");
        let help = "RORI API:
        nothing for now...";
        Ok(Response::with((status::Ok, help)))
    }

    // TODO add TLS
}
