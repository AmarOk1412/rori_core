extern crate core;
extern crate dbus;
extern crate rusqlite;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate time;

mod mocks;

#[cfg(test)]
mod tests_server {
    use core::rori::account::Account;
    use core::rori::database::Database;
    use core::rori::interaction::Interaction;
    use core::rori::server::Server;
    use core::rori::user::User;
    use mocks::Daemon;
    use rusqlite;
    use serde_json;
    use std::collections::HashMap;
    use std::io::prelude::*;
    use std::fs;
    use std::fs::File;
    use std::sync::{Arc, Mutex};
    use std::sync::atomic::Ordering;
    use std::thread;
    use std::time::Duration;
    use time;

    /**
     * ConfigFile structure
     * TBD
     */
    #[derive(Serialize, Deserialize)]
    pub struct ConfigFile {
        ring_id: String,
        api_listener: String,
        cert_path: String,
        cert_pass: String,
    }

    fn setup(anonymous: User, registered_users: Vec<User>) -> Server {
        let _ = fs::remove_file("rori.db");
        Database::init_db();
        // Insert modules
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        // NOTE: if much modules, launch generate_modules.py
        let mut req = conn.prepare("INSERT INTO modules (name, priority, enabled, type, condition, path)
                                    VALUES (\"history\", 0, 1, \"text\", \".*\", \"history\")").unwrap();
        let _ = req.execute(&[]);
        let mut req = conn.prepare("INSERT INTO modules (name, priority, enabled, type, condition, path)
                                    VALUES (\"hello_world\", 1, 1, \"text\", \"^(salut|bonjour|bonsoir|hei|hi|hello|yo|o/)( rori| ?!?)$\", \"talk/hello_world\")").unwrap();
        let _ = req.execute(&[]);
        let mut req = conn.prepare("INSERT INTO modules (name, priority, enabled, type, condition, path)
                                    VALUES (\"name\", 2, 1, \"text\", \"name\", \"talk/name\")").unwrap();
        let _ = req.execute(&[]);
        let mut req = conn.prepare("INSERT INTO modules (name, priority, enabled, type, condition, path)
                                    VALUES (\"name_duplicate\", 3, 1, \"text\", \"name\", \"talk/name\")").unwrap();
        let _ = req.execute(&[]);

        let account = Account {
            id: String::from("GLaDOs_id"),
            ring_id: String::from("GLaDOs_ring_id"),
            alias: String::from("GLaDOs_alias"),
            enabled: true,
        };
        let mut server = Server::new(account);
        server.registered_users = registered_users;
        server.anonymous_user = anonymous;

        let config = ConfigFile {
            ring_id: String::from("GLaDOs_id"),
            api_listener: String::new(),
            cert_path: String::new(),
            cert_pass: String::new()
        };
        let config = serde_json::to_string_pretty(&config).unwrap_or(String::new());
        let mut file = File::create("config.json").ok().expect("config.json found.");

        let _ = file.write_all(config.as_bytes());


        server
    }

    fn teardown() {
        let _ = fs::remove_file("rori.db");
        let _ = fs::remove_file("config.json");
    }

    #[test]
    // Scenario:
    // Retrieve some modules from the database
    fn database_get_enabled_modules() {
        setup(User::new(), Vec::new());
        let modules = Database::get_enabled_modules(0);
        assert!(modules.len() == 1);
        assert!(modules.first().unwrap().name == "history");
        let modules = Database::get_enabled_modules(1);
        assert!(modules.len() == 1);
        assert!(modules.first().unwrap().name == "hello_world");
        let modules = Database::get_enabled_modules(10);
        assert!(modules.len() == 0);
        teardown();
    }

    #[test]
    // Scenario:
    // Retrieve priorities
    fn database_get_descending_priorities() {
        setup(User::new(), Vec::new());
        let priorities = Database::get_descending_priorities();
        assert!(priorities.len() == 4);
        assert!(*priorities.get(0).unwrap() == 0);
        assert!(*priorities.get(1).unwrap() == 1);
        teardown();
    }

    #[test]
    // Scenario
    // 1. If a new message is received, should be stored in the history table via the history module
    fn modules_test_history() {
        let daemon = Arc::new(Mutex::new(Daemon::new()));
        let cloned_daemon = daemon.clone();
        let daemon_thread = thread::spawn(move|| {
            Daemon::run(cloned_daemon);
        });
        let mut server = setup(User::new(), Vec::new());
        // handle interaction from unknown device
        server.handle_interaction(Interaction {
            author_ring_id: String::from("Tars_id"),
            body: String::from("My joke percentage is at 70%!"),
            datatype: String::from("text/plain"),
            time: time::now(),
            metadatas: HashMap::new()
        });
        // Let the time to the module
        let fhundred_millis = Duration::from_millis(500);
        thread::sleep(fhundred_millis);

        // Should be present
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let mut stmt = conn.prepare("SELECT * FROM History").unwrap();
        let mut rows = stmt.query(&[]).unwrap();
        let mut nbrows = 0;
        while let Some(_) = rows.next() {
            nbrows += 1;
        }
        assert!(nbrows == 1);

        // handle interaction from unknown device
        server.handle_interaction(Interaction {
            author_ring_id: String::from("Tars_id"),
            body: String::from("My joke percentage is at 60%!"),
            datatype: String::from("text/plain"),
            time: time::now(),
            metadatas: HashMap::new()
        });
        // Let the time to the module
        let fhundred_millis = Duration::from_millis(500);
        thread::sleep(fhundred_millis);

        // Should be present
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let mut stmt = conn.prepare("SELECT * FROM History").unwrap();
        let mut rows = stmt.query(&[]).unwrap();
        let mut nbrows = 0;
        while let Some(_) = rows.next() {
            nbrows += 1;
        }
        assert!(nbrows == 2);

        teardown();
        daemon.lock().unwrap().stop();
        let _ = daemon_thread.join();
    }

    #[test]
    // Scenario
    // 1. Someone say yo. RORI should answer
    fn modules_test_hello_world() {
        let daemon = Arc::new(Mutex::new(Daemon::new()));
        let cloned_daemon = daemon.clone();
        let daemon_thread = thread::spawn(move|| {
            Daemon::run(cloned_daemon);
        });
        let mut server = setup(User::new(), Vec::new());
        // Tars_id do a yo
        server.handle_interaction(Interaction {
            author_ring_id: String::from("Tars_id"),
            body: String::from("yo"),
            datatype: String::from("text/plain"),
            time: time::now(),
            metadatas: HashMap::new()
        });

        // This should has sent 1 message
        let mut idx_signal = 0;
        let hundred_millis = Duration::from_millis(100);
        while idx_signal < 10 {
            let storage = daemon.lock().unwrap().storage.clone();
            let has_new_info = storage.lock().unwrap().new_info.load(Ordering::SeqCst);
            if has_new_info {
                let interactions = storage.lock().unwrap().interactions_sent.clone();
                assert!(interactions.len() == 1);
                break;
            }
            thread::sleep(hundred_millis);
            idx_signal += 1;
            if idx_signal == 10 {
                panic!("interactions not set!");
            }
        }
        teardown();
        daemon.lock().unwrap().stop();
        let _ = daemon_thread.join();
    }

    #[test]
    // Scenario
    // 1. Someone say name this should trigger only the first module and stop.
    fn modules_test_stop_priorities() {
        let daemon = Arc::new(Mutex::new(Daemon::new()));
        let cloned_daemon = daemon.clone();
        let daemon_thread = thread::spawn(move|| {
            Daemon::run(cloned_daemon);
        });
        let mut server = setup(User::new(), Vec::new());
        // Tars_id says name
        server.handle_interaction(Interaction {
            author_ring_id: String::from("PBody_id"),
            body: String::from("name"),
            datatype: String::from("text/plain"),
            time: time::now(),
            metadatas: HashMap::new()
        });

        // This should has sent 1 message (not 2!)
        let mut idx_signal = 0;
        let hundred_millis = Duration::from_millis(100);
        while idx_signal < 10 {
            let storage = daemon.lock().unwrap().storage.clone();
            let has_new_info = storage.lock().unwrap().new_info.load(Ordering::SeqCst);
            if has_new_info {
                let interactions = storage.lock().unwrap().interactions_sent.clone();
                assert!(interactions.len() == 1);
                break;
            }
            thread::sleep(hundred_millis);
            idx_signal += 1;
            if idx_signal == 10 {
                panic!("interactions not set!");
            }
        }
        teardown();
        daemon.lock().unwrap().stop();
        let _ = daemon_thread.join();
    }

    // NOTE: modules will not be tested here. But related code in rust files should be tested.
    // Last two tests test the module activation's loop
}
