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
                author_ring_id: String::from("Weasley"),
                body: String::from("/register weasley"),
                datatype: String::from("rori/command"),
                time: time::now(),
                metadatas: HashMap::new()
            });
            m.lock().unwrap().server.handle_interaction(Interaction {
                author_ring_id: String::from("Weasley"),
                body: String::from("/add_device core"),
                datatype: String::from("rori/command"),
                time: time::now(),
                metadatas: HashMap::new()
            });
            let mut api = API::new(m, String::from("0.0.0.0:1412"),
                                   String::from("./test_keys/api.p12"), String::new());
            api.start();
        });

        let three_secs = Duration::from_millis(3000);
        thread::sleep(three_secs);

        let client = reqwest::ClientBuilder::new()
                    .danger_accept_invalid_certs(true)
                    .build().unwrap();

        let mut res = match client.get("https://127.0.0.1:1412/name/rori").send() {
            Ok(res) => res,
            _ => {
                panic!("Can't get good result from API");
            }
        };
        let mut body: String = String::new();
        let _ = res.read_to_string(&mut body);
        let v: Value = from_str(&body).unwrap();
        assert!(v["name"] == "rori");
        assert!(v["addr"] == "0xGLaDOs_ring_id");

        let mut res = match client.get("https://127.0.0.1:1412/name/weasley").send() {
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

        let mut res = match client.get("https://127.0.0.1:1412/name/weasley_core").send() {
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

        let mut res = match client.get("https://127.0.0.1:1412/name/eve").send() {
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
                author_ring_id: String::from("weasley_id"),
                body: String::from("/register weasley"),
                datatype: String::from("rori/command"),
                time: time::now(),
                metadatas: HashMap::new(),
            });
            let mut api = API::new(m, String::from("0.0.0.0:1413"),
                                   String::from("./test_keys/api.p12"), String::new());
            api.start();
        });

        let three_secs = Duration::from_millis(3000);
        thread::sleep(three_secs);

        let client = reqwest::ClientBuilder::new()
                    .danger_accept_invalid_certs(true)
                    .build().unwrap();

        let mut res = match client.get("https://127.0.0.1:1413/addr/weasley_id").send() {
            Ok(res) => res,
            _ => {
                panic!("Can't get good result from API");
            }
        };
        let mut body: String = String::new();
        let _ = res.read_to_string(&mut body);
        let v: Value = from_str(&body).unwrap();
        assert!(v["name"] == "weasley");

        let mut res = match client.get("https://127.0.0.1:1413/addr/eve").send() {
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
}
