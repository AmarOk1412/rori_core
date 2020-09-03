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
use rori::database::Database;
use serde_json;
use std::sync::{Arc, Mutex};

/**
 * Publicly accessible to manipulate RORI from HTTP requests
 * Features:
 * + Ring compatible name server
 * TBD
 */
pub struct API {
    address: String,
    manager: Arc<Mutex<Manager>>
}

impl API {
    /**
     * Initializes the API
     * @param manager to access to RORI informations
     * @param address where the server listens
     * @param cert_path where the cert path is located
     * @param cert_pass
     * @return an API structure
     */
    pub fn new(manager: Arc<Mutex<Manager>>, address: String) -> API {
        API {
            address: address,
            manager: manager
        }
    }

    /**
     * Launch an API instance
     * @param self
     */
    pub fn start(&mut self) {
        let mut router = Router::new();
        // Init routes
        let name_handler = NameHandler {
            manager: self.manager.clone()
        };
        let addr_handler = AddrHandler { };

        router.get("/name/:name", name_handler, "name");
        router.get("/addr/:addr", addr_handler, "addr");
        info!("start API endpoint at {}", self.address);
        // Start router
        Iron::new(router).http(&*self.address).unwrap();
    }
}

/**
 * Following classes are used for the RING compatible name server.
 * See documentation here:
 * https://git.jami.net/savoirfairelinux/ring-project/wikis/technical/Name-Server-Protocol
 * For now, only the name endpoint is usefull
 */
struct NameHandler {
    manager: Arc<Mutex<Manager>>
}

/**
 * Used if success name's query
 */
#[derive(Serialize, Deserialize)]
struct NameResponse {
    name: String,
    addr: String,
    full_devices: String,
    bridges_devices: String,
}

/**
 * Used if an error occurs
 */
#[derive(Serialize, Deserialize)]
struct NameError {
    error: String,
}

impl Handler for NameHandler {
    fn handle(&self, request: &mut Request) -> IronResult<Response> {
        let content_type = "application/json".parse::<Mime>().unwrap();
        let name = request.extensions.get::<Router>().unwrap().find("name").unwrap_or("");
        info!("GET /name/{}", name);
        // rori's name is reserved
        if name.to_lowercase() == "rori" {
            let rori_addr = &self.manager.lock().unwrap().server.account.ring_id;
            let addr = format!("0x{}", rori_addr.replace("ring:", ""));
            let answer = NameResponse {
                name: String::from(name),
                addr: addr,
                full_devices: String::new(),
                bridges_devices: String::new(),
            };
            let response = serde_json::to_string(&answer).unwrap_or(String::new());
            return Ok(Response::with((content_type, status::Ok, response)))
        }
        let all_devices = Database::get_devices();
        let mut devices = Vec::new();
        for (id, hash, username, devicename, is_bridge) in all_devices {
            if username == name || name == &*format!("{}_{}", username, devicename) {
                devices.push((id, hash, username, devicename, is_bridge))
            }
        }
        // Build the response
        if devices.len() > 0 {
            let mut is_first = true;
            let mut addr = String::new();
            let mut full_devices = String::new();
            let mut bridges_devices = String::new();
            for (_, h, _, _, ib) in devices {
                if is_first {
                    is_first = false;
                    addr = format!("0x{}", h.replace("ring:", ""));
                }
                if ib {
                    bridges_devices += &*h;
                    bridges_devices += ";";
                } else {
                    full_devices += &*h;
                    full_devices += ";";
                }
            }
            let answer = NameResponse {
                name: String::from(name),
                addr: addr,
                full_devices: full_devices,
                bridges_devices: bridges_devices,
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

struct AddrHandler { }

/**
 * Used if success addr's query
 */
#[derive(Serialize, Deserialize)]
struct AddrResponse {
    name: String, // The first user
    is_bridge: bool,
    users_list: String
}

/**
 * Used if an error occurs
 */
#[derive(Serialize, Deserialize)]
struct AddrError {
    error: String,
}

impl Handler for AddrHandler {
    fn handle(&self, request: &mut Request) -> IronResult<Response> {
        let content_type = "application/json".parse::<Mime>().unwrap();
        let ring_id = request.extensions.get::<Router>().unwrap().find("addr").unwrap_or("");
        info!("GET /addr/{}", ring_id);

        // get usernames
        let devices = Database::get_devices_for_hash(&String::from(ring_id));
        let mut username = String::new();
        let mut is_bridge = false;
        let mut users_list = String::new();

        // Build the response
        if devices.len() > 0 {
            let mut is_first = true;
            for (_, _, u, _, ib) in devices {
                if is_first && u.len() > 0 {
                    is_first = false;
                    username = u.clone();
                    is_bridge = ib;
                }
                if u.len() > 0 {
                    users_list += &*u;
                    users_list += ";";
                }
            }

            let answer = AddrResponse {
                name: username,
                is_bridge: is_bridge,
                users_list: users_list,
            };
            let response = serde_json::to_string(&answer).unwrap_or(String::new());
            return Ok(Response::with((content_type, status::Ok, response)))
        }
        let answer = NameError {
            error: String::from("address not registred")
        };
        let response = serde_json::to_string(&answer).unwrap_or(String::new());
        Ok(Response::with((content_type, status::NotFound, response)))
    }
}
