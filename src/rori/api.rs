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
use rori::scheduler::Scheduler;
use rori::database::Database;
use serde_json;
use std::collections::HashMap;
use std::io::Read;
use std::sync::{Arc, Mutex};

/**
 * Publicly accessible to manipulate RORI from HTTP requests
 * Features:
 * + Ring compatible name server
 * TBD
 */
pub struct API {
    address: String,
    scheduler: Arc<Mutex<Scheduler>>,
    manager: Arc<Mutex<Manager>>,
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
            scheduler: Arc::new(Mutex::new(Scheduler::new())),
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
        let task_add_handler = TaskAddHandler {
            scheduler: self.scheduler.clone()
        };
        let task_update_handler = TaskUpdateHandler {
            scheduler: self.scheduler.clone()
        };
        let task_rm_handler = TaskRmHandler {
            scheduler: self.scheduler.clone()
        };
        let task_search_handler = TaskSearchHandler { };
        let module_handler = ModuleHandler { };

        router.get("/name/:name", name_handler, "name");
        router.get("/addr/:addr", addr_handler, "addr");
        // POST task/add {JSON}
        router.post("task/add", task_add_handler, "task_add");
        // POST task/update {JSON}
        router.post("task/update", task_update_handler, "task_update");
        // DELETE task/id
        router.delete("task/:id", task_rm_handler, "task_rm");
        // POST task/search/name {JSON}
        router.post("/task/search/:name", task_search_handler, "task_search");
        // GET module/name
        router.get("/module/:name", module_handler, "module");
        info!("start API endpoint at {}", self.address);
        // Start router
        Iron::new(router).http(&*self.address).unwrap();
    }
}

/**
 * Following classes are used for the Jami compatible name server.
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

/**
 * Used to add tasks to the scheduler
 */
struct TaskAddHandler {
    scheduler: Arc<Mutex<Scheduler>>,
}

/**
 * Used if task is added with success
 */
#[derive(Serialize, Deserialize)]
struct TaskAddResponse {
    id: i32,
}

/**
 * Used if an error occurs
 */
#[derive(Serialize, Deserialize)]
struct TaskAddError {
    error: String,
}

impl Handler for TaskAddHandler {
    fn handle(&self, request: &mut Request) -> IronResult<Response> {
        let content_type = "application/json".parse::<Mime>().unwrap();
        let mut body = String::new();
        request.body.read_to_string(&mut body).unwrap();
        info!("POST /task/add {}", body);
        let result = self.scheduler.lock().unwrap().add_task(&body);
        match result {
            Some(result) => {
                let answer = TaskAddResponse { id: result };
                let response = serde_json::to_string(&answer).unwrap_or(String::new());
                return Ok(Response::with((content_type, status::Ok, response)));
            },
            _ => {
                let answer = TaskAddError { error: String::from("Could not add task" )};
                let response = serde_json::to_string(&answer).unwrap_or(String::new());
                return Ok(Response::with((content_type, status::NotFound, response)));
            }
        };
    }
}

/**
 * Used to update tasks to the scheduler
 */
struct TaskUpdateHandler {
    scheduler: Arc<Mutex<Scheduler>>,
}

/**
 * Used if the task is updated with success
 */
#[derive(Serialize, Deserialize)]
struct TaskUpdateResponse {
    id: i32,
}

/**
 * Used if an error occurs
 */
#[derive(Serialize, Deserialize)]
struct TaskUpdateError {
    error: String,
}

impl Handler for TaskUpdateHandler {
    fn handle(&self, request: &mut Request) -> IronResult<Response> {
        let content_type = "application/json".parse::<Mime>().unwrap();
        let mut body = String::new();
        request.body.read_to_string(&mut body).unwrap();
        info!("POST /task/update {}", body);
        let result = self.scheduler.lock().unwrap().update_task(&body);
        match result {
            Some(result) => {
                let answer = TaskUpdateResponse { id: result };
                let response = serde_json::to_string(&answer).unwrap_or(String::new());
                return Ok(Response::with((content_type, status::Ok, response)));
            },
            _ => {
                let answer = TaskUpdateError { error: String::from("Could not update task" )};
                let response = serde_json::to_string(&answer).unwrap_or(String::new());
                return Ok(Response::with((content_type, status::NotFound, response)));
            }
        };
    }
}

/**
 * Used to remove a task from the scheduler
 */
struct TaskRmHandler {
    scheduler: Arc<Mutex<Scheduler>>,
}

/**
 * Used if the task is deleted with success
 */
#[derive(Serialize, Deserialize)]
struct TaskRmResponse {
    id: i32,
}

/**
 * Used if an error occurs
 */
#[derive(Serialize, Deserialize)]
struct TaskRmError {
    error: String,
}

impl Handler for TaskRmHandler {
    fn handle(&self, request: &mut Request) -> IronResult<Response> {
        let content_type = "application/json".parse::<Mime>().unwrap();
        let id = request.extensions.get::<Router>().unwrap().find("id").unwrap_or("").parse::<i32>().unwrap_or(0);
        info!("DELETE /task/{}", id);
        let result = self.scheduler.lock().unwrap().rm_task(&id);
        match result {
            Some(result) => {
                let answer = TaskRmResponse { id: result };
                let response = serde_json::to_string(&answer).unwrap_or(String::new());
                return Ok(Response::with((content_type, status::Ok, response)));
            },
            _ => {
                let answer = TaskRmError { error: String::from("Could not remove task" )};
                let response = serde_json::to_string(&answer).unwrap_or(String::new());
                return Ok(Response::with((content_type, status::NotFound, response)));
            }
        };
    }
}

/**
 * Used to search a task
 */
struct TaskSearchHandler { }

/**
 * Used if the task is found
 */
#[derive(Serialize, Deserialize)]
struct TaskSearchResponse {
    id: i32,
}

/**
 * Used if an error occurs
 */
#[derive(Serialize, Deserialize)]
struct TaskSearchError {
    error: String,
}

impl Handler for TaskSearchHandler {
    fn handle(&self, request: &mut Request) -> IronResult<Response> {
        let content_type = "application/json".parse::<Mime>().unwrap();
        let name = request.extensions.get::<Router>().unwrap().find("name").unwrap_or("");
        let mut body = String::new();
        request.body.read_to_string(&mut body).unwrap();
        info!("POST /task/search/{} {}", name, body);

        let content: HashMap<String, String> = serde_json::from_str(&*body).unwrap_or(HashMap::new());
        let result = Database::search_task(&String::from(name), content);
        match result {
            Some(result) => {
                let answer = ModuleResponse { id: result.id };
                let response = serde_json::to_string(&answer).unwrap_or(String::new());
                return Ok(Response::with((content_type, status::Ok, response)));
            },
            _ => {
                let answer = ModuleError { error: String::from("Could not get module" )};
                let response = serde_json::to_string(&answer).unwrap_or(String::new());
                return Ok(Response::with((content_type, status::NotFound, response)));
            }
        };
    }
}

/**
 * Used to search a module
 */
struct ModuleHandler { }

/**
 * Used if success Module's query
 */
#[derive(Serialize, Deserialize)]
struct ModuleResponse {
    id: i32,
}

/**
 * Used if an error occurs
 */
#[derive(Serialize, Deserialize)]
struct ModuleError {
    error: String,
}

impl Handler for ModuleHandler {
    fn handle(&self, request: &mut Request) -> IronResult<Response> {
        let content_type = "application/json".parse::<Mime>().unwrap();
        let name = request.extensions.get::<Router>().unwrap().find("name").unwrap_or("");
        info!("GET /module/{}", name);

        let result = Database::get_module_id_by_name(&String::from(name));
        match result {
            0 => {
                let answer = ModuleError { error: String::from("Could not get module" )};
                let response = serde_json::to_string(&answer).unwrap_or(String::new());
                return Ok(Response::with((content_type, status::NotFound, response)));
            }
            result => {
                let answer = ModuleResponse { id: result };
                let response = serde_json::to_string(&answer).unwrap_or(String::new());
                return Ok(Response::with((content_type, status::Ok, response)));
            },
        };
    }
}
