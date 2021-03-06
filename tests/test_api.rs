extern crate core;
extern crate dbus;
extern crate serde_json;
extern crate reqwest;
extern crate time;

mod mocks;
#[cfg(test)]
mod tests_api {
    use core::rori::api::API;
    use core::rori::database::Database;
    use core::rori::interaction::Interaction;
    use core::rori::manager::Manager;
    use core::rori::user::Device;
    use mocks::Daemon;
    use reqwest;
    use serde_json::{Value, from_str};
    use std::collections::HashMap;
    use std::fs;
    use std::io::Read;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use std::thread;
    use time;

    fn setup() {
        let _ = fs::remove_file("rori.db");
        Database::init_db(); // assert this function is correct.
    }

    fn teardown() {
        let _ = fs::remove_file("rori.db");
    }

    #[test]
    // Scenario
    // 1. get /name/rori
    // 2. get /name/weasley where weasley is a registered User
    // 3. get /name/weasley_core where weasley_core is a registered Device
    // 4. get /name/eve where eve is unknown
    fn api_get_name() {
        setup();
        let daemon = Arc::new(Mutex::new(Daemon::new()));
        let cloned_daemon = daemon.clone();
        let daemon_thread = thread::spawn(move|| {
            Daemon::run(cloned_daemon);
        });
        let _ = thread::spawn(move|| {
            let m = Arc::new(Mutex::new(Manager::init("GLaDOs_id").unwrap()));
            m.lock().unwrap().server.handle_interaction(Interaction {
                device_author: Device {
                    id: 0,
                    name: String::new(),
                    ring_id: String::from("Weasley"),
                    is_bridge: false
                },
                body: String::from("/register weasley"),
                datatype: String::from("rori/command"),
                time: time::now(),
                metadatas: HashMap::new()
            });
            m.lock().unwrap().server.handle_interaction(Interaction {
                device_author: Device {
                    id: 0,
                    name: String::new(),
                    ring_id: String::from("Weasley"),
                    is_bridge: false
                },
                body: String::from("/add_device core"),
                datatype: String::from("rori/command"),
                time: time::now(),
                metadatas: HashMap::new()
            });
            let mut api = API::new(m, String::from("0.0.0.0:1412"));
            api.start();
        });

        let three_secs = Duration::from_millis(3000);
        thread::sleep(three_secs);

        let client = reqwest::ClientBuilder::new()
                    .danger_accept_invalid_certs(true)
                    .build().unwrap();

        let mut res = match client.get("http://127.0.0.1:1412/name/rori").send() {
            Ok(res) => res,
            _ => {
                panic!("Can't get good result from API");
            }
        };
        let mut body: String = String::new();
        let _ = res.read_to_string(&mut body);
        let v: Value = from_str(&body).unwrap();
        assert!(v["name"] == "rori");
        assert!(v["addr"] == "0xGLaDOs_hash");

        let mut res = match client.get("http://127.0.0.1:1412/name/weasley").send() {
            Ok(res) => res,
            _ => {
                panic!("Can't get good result from API");
            }
        };
        let mut body: String = String::new();
        let _ = res.read_to_string(&mut body);
        let v: Value = from_str(&body).unwrap();
        assert!(v["name"] == "weasley");
        assert!(v["addr"] == "0xWeasley");

        let mut res = match client.get("http://127.0.0.1:1412/name/weasley_core").send() {
            Ok(res) => res,
            _ => {
                panic!("Can't get good result from API");
            }
        };
        let mut body: String = String::new();
        let _ = res.read_to_string(&mut body);
        let v: Value = from_str(&body).unwrap();
        assert!(v["name"] == "weasley_core");
        assert!(v["addr"] == "0xWeasley");

        let mut res = match client.get("http://127.0.0.1:1412/name/eve").send() {
            Ok(res) => res,
            _ => {
                panic!("Can't get good result from API");
            }
        };
        let mut body: String = String::new();
        let _ = res.read_to_string(&mut body);
        let v: Value = from_str(&body).unwrap();
        assert!(v["error"] == "name not registred");

        daemon.lock().unwrap().stop();
        let _ = daemon_thread.join();
        teardown();
    }

    #[test]
    // Scenario
    // 1. get /addr/user_id
    // 2. get /addr/not_user_id
    fn api_get_addr() {
        setup();
        let daemon = Arc::new(Mutex::new(Daemon::new()));
        let cloned_daemon = daemon.clone();
        let daemon_thread = thread::spawn(move|| {
            Daemon::run(cloned_daemon);
        });
        let _ = thread::spawn(move|| {
            let m = Arc::new(Mutex::new(Manager::init("GLaDOs_id").unwrap()));
            m.lock().unwrap().server.handle_interaction(Interaction {
                device_author: Device {
                    id: 0,
                    name: String::new(),
                    ring_id: String::from("weasley_id"),
                    is_bridge: false
                },
                body: String::from("/register weasley"),
                datatype: String::from("rori/command"),
                time: time::now(),
                metadatas: HashMap::new(),
            });
            let mut api = API::new(m, String::from("0.0.0.0:1413"));
            api.start();
        });

        let three_secs = Duration::from_millis(3000);
        thread::sleep(three_secs);

        let client = reqwest::ClientBuilder::new()
                    .danger_accept_invalid_certs(true)
                    .build().unwrap();

        let mut res = match client.get("http://127.0.0.1:1413/addr/weasley_id").send() {
            Ok(res) => res,
            _ => {
                panic!("Can't get good result from API");
            }
        };
        let mut body: String = String::new();
        let _ = res.read_to_string(&mut body);
        let v: Value = from_str(&body).unwrap();
        assert!(v["name"] == "weasley");

        let mut res = match client.get("http://127.0.0.1:1413/addr/eve").send() {
            Ok(res) => res,
            _ => {
                panic!("Can't get good result from API");
            }
        };
        let mut body: String = String::new();
        let _ = res.read_to_string(&mut body);
        let v: Value = from_str(&body).unwrap();
        assert!(v["error"] == "address not registred");

        daemon.lock().unwrap().stop();
        let _ = daemon_thread.join();
        teardown();
    }

    #[test]
    // Scenario
    // 1. get /name/rori
    // 2. get /name/weasley where weasley is a registered User from bridge
    fn api_get_name_with_bridges() {
        setup();
        let daemon = Arc::new(Mutex::new(Daemon::new()));
        let cloned_daemon = daemon.clone();
        let daemon_thread = thread::spawn(move|| {
            Daemon::run(cloned_daemon);
        });
        let _ = thread::spawn(move|| {
            let m = Arc::new(Mutex::new(Manager::init("GLaDOs_id").unwrap()));
            m.lock().unwrap().server.handle_interaction(Interaction {
                device_author: Device {
                    id: 1,
                    name: String::new(),
                    ring_id: String::from("Weasley"),
                    is_bridge: false
                },
                body: String::from("/bridgify"),
                datatype: String::from("rori/command"),
                time: time::now(),
                metadatas: HashMap::new()
            });
            m.lock().unwrap().server.handle_interaction(Interaction {
                device_author: Device {
                    id: 1,
                    name: String::new(),
                    ring_id: String::from("Weasley"),
                    is_bridge: false
                },
                body: String::from("/register weasley"),
                datatype: String::from("rori/command"),
                time: time::now(),
                metadatas: HashMap::new()
            });
            m.lock().unwrap().server.handle_interaction(Interaction {
                device_author: Device {
                    id: 1,
                    name: String::new(),
                    ring_id: String::from("Weasley"),
                    is_bridge: false
                },
                body: String::from("/add_device core"),
                datatype: String::from("rori/command"),
                time: time::now(),
                metadatas: HashMap::new()
            });
            let mut api = API::new(m, String::from("0.0.0.0:1414"));
            api.start();
        });

        let three_secs = Duration::from_millis(3000);
        thread::sleep(three_secs);

        let client = reqwest::ClientBuilder::new()
                    .danger_accept_invalid_certs(true)
                    .build().unwrap();

        let mut res = match client.get("http://127.0.0.1:1414/name/rori").send() {
            Ok(res) => res,
            _ => {
                panic!("Can't get good result from API");
            }
        };
        let mut body: String = String::new();
        let _ = res.read_to_string(&mut body);
        let v: Value = from_str(&body).unwrap();
        assert!(v["name"] == "rori");
        assert!(v["addr"] == "0xGLaDOs_hash");
        assert!(v["bridges_devices"] == "");

        let mut res = match client.get("http://127.0.0.1:1414/name/weasley").send() {
            Ok(res) => res,
            _ => {
                panic!("Can't get good result from API");
            }
        };
        let mut body: String = String::new();
        let _ = res.read_to_string(&mut body);
        let v: Value = from_str(&body).unwrap();
        assert!(v["name"] == "weasley");
        assert!(v["addr"] == "0xWeasley");
        assert!(v["bridges_devices"] == "Weasley;");

        daemon.lock().unwrap().stop();
        let _ = daemon_thread.join();
        teardown();
    }

    #[test]
    fn api_get_module() {
        setup();
        // Insert module foo
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let _ = conn.execute("INSERT INTO modules (name, priority, enabled, type, condition, path)
                                VALUES (\"foo\", 1, 1, \"foo\", \"foo\", \"foo\")", rusqlite::NO_PARAMS);

        let daemon = Arc::new(Mutex::new(Daemon::new()));
        let cloned_daemon = daemon.clone();
        let daemon_thread = thread::spawn(move|| {
            Daemon::run(cloned_daemon);
        });
        let _ = thread::spawn(move|| {
            let m = Arc::new(Mutex::new(Manager::init("GLaDOs_id").unwrap()));
            m.lock().unwrap().server.handle_interaction(Interaction {
                device_author: Device {
                    id: 0,
                    name: String::new(),
                    ring_id: String::from("Weasley"),
                    is_bridge: false
                },
                body: String::from("/register weasley"),
                datatype: String::from("rori/command"),
                time: time::now(),
                metadatas: HashMap::new()
            });
            m.lock().unwrap().server.handle_interaction(Interaction {
                device_author: Device {
                    id: 0,
                    name: String::new(),
                    ring_id: String::from("Weasley"),
                    is_bridge: false
                },
                body: String::from("/add_device core"),
                datatype: String::from("rori/command"),
                time: time::now(),
                metadatas: HashMap::new()
            });
            let mut api = API::new(m, String::from("0.0.0.0:1416"));
            api.start();
        });

        let three_secs = Duration::from_millis(3000);
        thread::sleep(three_secs);

        let client = reqwest::ClientBuilder::new()
                    .danger_accept_invalid_certs(true)
                    .build().unwrap();

        let mut res = match client.get("http://127.0.0.1:1416/module/foo").send() {
            Ok(res) => res,
            _ => {
                panic!("Can't get good result from API");
            }
        };
        let mut body: String = String::new();
        let _ = res.read_to_string(&mut body);
        let v: Value = from_str(&body).unwrap();
        assert!(v["id"] == 1);

        let mut res = match client.get("http://127.0.0.1:1416/module/bar").send() {
            Ok(res) => res,
            _ => {
                panic!("Can't get good result from API");
            }
        };
        let mut body: String = String::new();
        let _ = res.read_to_string(&mut body);
        let v: Value = from_str(&body).unwrap();
        assert!(!v.get("error").unwrap().to_string().is_empty());

        daemon.lock().unwrap().stop();
        let _ = daemon_thread.join();
        teardown();
    }

    #[test]
    fn api_scheduler() {
        setup();
        // Insert module foo
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let _ = conn.execute("INSERT INTO modules (name, priority, enabled, type, condition, path)
                                VALUES (\"foo\", 1, 1, \"foo\", \"foo\", \"foo\")", rusqlite::NO_PARAMS);
        let _ = Database::insert_new_device(&String::from("Weasley"), &String::from("Weasley"), &String::from("Weasley"), false);

        let daemon = Arc::new(Mutex::new(Daemon::new()));
        let cloned_daemon = daemon.clone();
        let daemon_thread = thread::spawn(move|| {
            Daemon::run(cloned_daemon);
        });
        let _ = thread::spawn(move|| {
            let m = Arc::new(Mutex::new(Manager::init("GLaDOs_id").unwrap()));
            m.lock().unwrap().server.handle_interaction(Interaction {
                device_author: Device {
                    id: 0,
                    name: String::new(),
                    ring_id: String::from("Weasley"),
                    is_bridge: false
                },
                body: String::from("/register weasley"),
                datatype: String::from("rori/command"),
                time: time::now(),
                metadatas: HashMap::new()
            });
            m.lock().unwrap().server.handle_interaction(Interaction {
                device_author: Device {
                    id: 0,
                    name: String::new(),
                    ring_id: String::from("Weasley"),
                    is_bridge: false
                },
                body: String::from("/add_device core"),
                datatype: String::from("rori/command"),
                time: time::now(),
                metadatas: HashMap::new()
            });
            let mut api = API::new(m, String::from("0.0.0.0:1417"));
            api.start();
        });

        let three_secs = Duration::from_millis(3000);
        thread::sleep(three_secs);

        let mut task = HashMap::new();
        task.insert(String::from("id"), String::from("0"));
        task.insert(String::from("module"), String::from("1"));
        task.insert(String::from("parameter"), String::from("{\"ring_id\":\"Weasley\",\"username\":\"Weasley\"}"));
        task.insert(String::from("at"), String::new());
        task.insert(String::from("seconds"), String::from("0"));
        task.insert(String::from("minutes"), String::from("0"));
        task.insert(String::from("hours"), String::from("0"));
        task.insert(String::from("days"), String::new());
        task.insert(String::from("repeat"), String::from("True"));
        let mut itask = HashMap::new();
        itask.insert(String::from("id"), String::from("0"));
        itask.insert(String::from("module"), String::from("1"));
        itask.insert(String::from("at"), String::new());
        itask.insert(String::from("seconds"), String::from("0"));
        itask.insert(String::from("minutes"), String::from("0"));
        itask.insert(String::from("hours"), String::from("0"));
        itask.insert(String::from("days"), String::new());
        itask.insert(String::from("repeat"), String::from("True"));

        let client = reqwest::ClientBuilder::new()
                    .danger_accept_invalid_certs(true)
                    .build().unwrap();

        // Valid add
        let mut res = match client.post("http://127.0.0.1:1417/task/add").json(&task).send() {
            Ok(res) => res,
            _ => {
                panic!("Can't get good result from API");
            }
        };
        let mut body: String = String::new();
        let _ = res.read_to_string(&mut body);
        let v: Value = from_str(&body).unwrap();
        assert!(v["id"] == 1);

        // Invalid add
        let mut res = match client.post("http://127.0.0.1:1417/task/add").json(&itask).send() {
            Ok(res) => res,
            _ => {
                panic!("Can't get good result from API");
            }
        };
        let mut body: String = String::new();
        let _ = res.read_to_string(&mut body);
        let v: Value = from_str(&body).unwrap();
        assert!(!v.get("error").unwrap().to_string().is_empty());

        // valid update
        task.insert(String::from("id"), String::from("1"));
        task.insert(String::from("minutes"), String::from("2"));
        let mut res = match client.post("http://127.0.0.1:1417/task/update").json(&task).send() {
            Ok(res) => res,
            _ => {
                panic!("Can't get good result from API");
            }
        };
        let mut body: String = String::new();
        let _ = res.read_to_string(&mut body);
        let v: Value = from_str(&body).unwrap();
        assert!(v["id"] == 1);

        // invalid update
        let mut res = match client.post("http://127.0.0.1:1417/task/update").json(&itask).send() {
            Ok(res) => res,
            _ => {
                panic!("Can't get good result from API");
            }
        };
        let mut body: String = String::new();
        let _ = res.read_to_string(&mut body);
        let v: Value = from_str(&body).unwrap();
        assert!(!v.get("error").unwrap().to_string().is_empty());

        let mut search_map = HashMap::new();

        // invalid search
        let mut res = match client.post("http://127.0.0.1:1417/task/search/GLaDOs").json(&search_map).send() {
            Ok(res) => res,
            _ => {
                panic!("Can't get good result from API");
            }
        };
        let mut body: String = String::new();
        let _ = res.read_to_string(&mut body);
        let v: Value = from_str(&body).unwrap();
        assert!(!v.get("error").unwrap().to_string().is_empty());

        // invalid valid search with no details
        let mut res = match client.post("http://127.0.0.1:1417/task/search/1").json(&search_map).send() {
            Ok(res) => res,
            _ => {
                panic!("Can't get good result from API");
            }
        };
        let mut body: String = String::new();
        let _ = res.read_to_string(&mut body);
        let v: Value = from_str(&body).unwrap();
        assert!(v["id"] == 1);

        // invalid valid search with details
        search_map.insert("ring_id", "Weasley");
        search_map.insert("username", "Weasley");
        let mut res = match client.post("http://127.0.0.1:1417/task/search/1").json(&search_map).send() {
            Ok(res) => res,
            _ => {
                panic!("Can't get good result from API");
            }
        };
        let mut body: String = String::new();
        let _ = res.read_to_string(&mut body);
        let v: Value = from_str(&body).unwrap();
        assert!(v["id"] == 1);

        // invalid valid search with too much details
        search_map.insert("love", "trahison");
        let mut res = match client.post("http://127.0.0.1:1417/task/search/1").json(&search_map).send() {
            Ok(res) => res,
            _ => {
                panic!("Can't get good result from API");
            }
        };
        let mut body: String = String::new();
        let _ = res.read_to_string(&mut body);
        let v: Value = from_str(&body).unwrap();
        assert!(!v.get("error").unwrap().to_string().is_empty());

        // invalid rm
        let mut res = match client.delete("http://127.0.0.1:1417/task/1412").send() {
            Ok(res) => res,
            _ => {
                panic!("Can't get good result from API");
            }
        };
        let mut body: String = String::new();
        let _ = res.read_to_string(&mut body);
        let v: Value = from_str(&body).unwrap();
        assert!(!v.get("error").unwrap().to_string().is_empty());

        // rm task
        let mut res = match client.delete("http://127.0.0.1:1417/task/1").send() {
            Ok(res) => res,
            _ => {
                panic!("Can't get good result from API");
            }
        };
        let mut body: String = String::new();
        let _ = res.read_to_string(&mut body);
        let v: Value = from_str(&body).unwrap();
        assert!(v["id"] == 1);

        daemon.lock().unwrap().stop();
        let _ = daemon_thread.join();
        teardown();
    }

}
