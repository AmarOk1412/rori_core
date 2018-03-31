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
    use std::fs;
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
                time: time::now()
            });
            m.lock().unwrap().server.handle_interaction(Interaction {
                author_ring_id: String::from("Weasley"),
                body: String::from("/add_device core"),
                time: time::now()
            });
            let mut api = API::new(m, String::from("0.0.0.0:1412"));
            api.start();
        });

        let three_secs = Duration::from_millis(3000);
        thread::sleep(three_secs);

        let body = reqwest::get("http://127.0.0.1:1412/name/rori").unwrap()
                            .text().unwrap();
        let v: Value = from_str(&body).unwrap();
        assert!(v["name"] == "rori");
        assert!(v["addr"] == "GLaDOs_ring_id");

        let body = reqwest::get("http://127.0.0.1:1412/name/weasley").unwrap()
                            .text().unwrap();
        let v: Value = from_str(&body).unwrap();
        assert!(v["name"] == "weasley");
        assert!(v["addr"] == "Weasley");

        let body = reqwest::get("http://127.0.0.1:1412/name/weasley_core").unwrap()
                            .text().unwrap();
        let v: Value = from_str(&body).unwrap();
        assert!(v["name"] == "weasley_core");
        assert!(v["addr"] == "Weasley");

        let body = reqwest::get("http://127.0.0.1:1412/name/eve").unwrap()
                            .text().unwrap();
        let v: Value = from_str(&body).unwrap();
        assert!(v["error"] == "name not registred");

        daemon.lock().unwrap().stop();
        let _ = daemon_thread.join();
        teardown();
    }
}
