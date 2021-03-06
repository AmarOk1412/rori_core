extern crate core;
extern crate dbus;
extern crate time;

mod mocks;
#[cfg(test)]
mod tests_manager {
    use core::rori::database::Database;
    use core::rori::manager::Manager;
    use mocks::Daemon;
    use std::fs;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use std::thread;

    fn setup() {
        let _ = fs::remove_file("rori.db");
        Database::init_db(); // assert this function is correct.
        let _ = Database::insert_new_device(&String::from("Atlas"), &String::new(), &String::new(), false);
        let _ = Database::insert_new_device(&String::from("Heisenberg"), &String::new(), &String::new(), false);
    }

    fn teardown() {
        let _ = fs::remove_file("rori.db");
    }

    #[test]
    // Scenario
    // 1. Ensures that a manager correctly enable and build an account
    fn manager_init() {
        setup();
        let daemon = Arc::new(Mutex::new(Daemon::new()));
        let cloned_daemon = daemon.clone();
        let daemon_thread = thread::spawn(move|| {
            Daemon::run(cloned_daemon);
        });
        let one_sec = Duration::from_millis(1000);
        thread::sleep(one_sec);

        // Test an account not here = Err(_)
        let m = Manager::init("Heisenberg");
        assert!(!m.is_ok());

        let m = Manager::init("GLaDOs_id");
        assert!(m.is_ok());
        // GLaDOs_id should be enabled now.
        let storage = daemon.lock().unwrap().storage.clone();
        let account = storage.lock().unwrap().account.clone();
        let account = account.lock().unwrap();
        assert!(account.enabled == true);
        assert!(account.id == "GLaDOs_id");

        // Ensures that contacts are in server and in db.
        assert!(Database::get_devices().len() == 4);
        let devices = m.unwrap().server.anonymous_user.devices;
        assert!(devices.len() == 4);
        match devices.iter().find(|d| d.ring_id == "Atlas") {
            Some(_) => {},
            None => { panic!("Atlas not found") }
        };
        match devices.iter().find(|d| d.ring_id == "PBody") {
            Some(_) => {},
            None => { panic!("PBody not found") }
        };
        match devices.iter().find(|d| d.ring_id == "Weasley") {
            Some(_) => {},
            None => { panic!("Weasley not found") }
        };
        match devices.iter().find(|d| d.ring_id == "Space core") {
            Some(_) => {},
            None => { panic!("Space core not found") }
        };

        daemon.lock().unwrap().stop();
        let _ = daemon_thread.join();
        teardown();
    }

    #[test]
    // Scenario
    // 1. Ensures that a manager correctly handle signal incomingTrustRequest
    fn manager_handle_signal_incoming_trust_request() {
        setup();
        let daemon = Arc::new(Mutex::new(Daemon::new()));
        let cloned_daemon = daemon.clone();
        let daemon_thread = thread::spawn(move|| {
            Daemon::run(cloned_daemon);
        });
        let one_sec = Duration::from_millis(1000);
        thread::sleep(one_sec);

        let shared_manager : Arc<Mutex<Manager>> = Arc::new(Mutex::new(
            Manager::init("GLaDOs_id")
            .ok().expect("Can't initialize ConfigurationManager"))
        );
        let shared_manager_cloned = shared_manager.clone();
        let stop = Arc::new(AtomicBool::new(false));
        let stop_cloned = stop.clone();
        let test = thread::spawn(move || {
            Manager::handle_signals(shared_manager_cloned, stop_cloned);
        });

        let two_sec = Duration::from_millis(5000);
        thread::sleep(two_sec);
        daemon.lock().unwrap().emit_incoming_trust_request();

        // Contact should be added daemon side
        let base_size = shared_manager.lock().unwrap().server.anonymous_user.devices.len();
        let mut idx_signal = 0;
        let hundred_millis = Duration::from_millis(100);
        while idx_signal < 100 {
            let storage = daemon.lock().unwrap().storage.clone();
            let accepted = storage.lock().unwrap().request_accepted.clone();
            let mut stop = false;
            for acc in accepted {
                if acc == "Eve" {
                    stop = true;
                    break;
                }
            }
            if stop {
                break;
            }
            thread::sleep(hundred_millis);
            idx_signal += 1;
        }

        // A new anonymous user (Eve) should be present
        assert!(shared_manager.lock().unwrap().server.anonymous_user.devices.len() == base_size + 1);
        let mut eve_found = false;
        for device in shared_manager.lock().unwrap().server.anonymous_user.devices.clone() {
            if device.ring_id == "Eve" {
                eve_found = true;
                break;
            }
        }
        assert!(eve_found);

        stop.store(true, Ordering::SeqCst);
        let _ = test.join();
        daemon.lock().unwrap().stop();
        let _ = daemon_thread.join();
        teardown();
    }

    #[test]
    // Scenario
    // 1. Ensures that a manager correctly handle signal incomingAccountMessage
    fn manager_handle_signal_incoming_account_message() {
        setup();
        let daemon = Arc::new(Mutex::new(Daemon::new()));
        let cloned_daemon = daemon.clone();
        let daemon_thread = thread::spawn(move|| {
            Daemon::run(cloned_daemon);
        });
        let one_sec = Duration::from_millis(1000);
        thread::sleep(one_sec);

        let shared_manager : Arc<Mutex<Manager>> = Arc::new(Mutex::new(
            Manager::init("GLaDOs_id")
            .ok().expect("Can't initialize ConfigurationManager"))
        );
        let shared_manager_cloned = shared_manager.clone();
        let stop = Arc::new(AtomicBool::new(false));
        let stop_cloned = stop.clone();
        let test = thread::spawn(move || {
            Manager::handle_signals(shared_manager_cloned, stop_cloned);
        });

        let two_sec = Duration::from_millis(5000);
        thread::sleep(two_sec);
        daemon.lock().unwrap().emit_incoming_account_message(&String::from("plain/text"), &String::from("Allo, I'm Eve!"));

        // Contact should be added daemon side
        let base_size = shared_manager.lock().unwrap().server.anonymous_user.devices.len();
        let mut idx_signal = 0;
        let hundred_millis = Duration::from_millis(100);
        while idx_signal < 100 {
            let storage = daemon.lock().unwrap().storage.clone();
            let accepted = storage.lock().unwrap().request_accepted.clone();
            let mut stop = false;
            for acc in accepted {
                if acc == "Eve" {
                    stop = true;
                    break;
                }
            }
            if stop {
                break;
            }
            thread::sleep(hundred_millis);
            idx_signal += 1;
        }

        // A new anonymous user (Eve) should be present
        assert!(shared_manager.lock().unwrap().server.anonymous_user.devices.len() == base_size + 1);
        let mut eve_found = false;
        for device in shared_manager.lock().unwrap().server.anonymous_user.devices.clone() {
            if device.ring_id == "Eve" {
                eve_found = true;
                break;
            }
        }
        assert!(eve_found);

        stop.store(true, Ordering::SeqCst);
        let _ = test.join();
        daemon.lock().unwrap().stop();
        let _ = daemon_thread.join();
        teardown();
    }

    #[test]
    // Scenario
    // 1. Retrieve account list
    fn manager_get_account_list() {
        setup();
        let daemon = Arc::new(Mutex::new(Daemon::new()));
        let cloned_daemon = daemon.clone();
        let daemon_thread = thread::spawn(move|| {
            Daemon::run(cloned_daemon);
        });
        let one_sec = Duration::from_millis(1000);
        thread::sleep(one_sec);

        let account_list = Manager::get_account_list();
        assert!(account_list.len() == 1);
        let account = account_list.first().unwrap();
        assert!(account.id == "GLaDOs_id");
        assert!(account.ring_id == "GLaDOs_hash");
        assert!(account.alias == "GLaDOs");
        assert!(!account.enabled);

        daemon.lock().unwrap().stop();
        let _ = daemon_thread.join();
        teardown();
    }

    #[test]
    // Scenario
    // 1. Add an account from a path
    // 2. Create a new account
    fn manager_add_account() {
        setup();
        let daemon = Arc::new(Mutex::new(Daemon::new()));
        let cloned_daemon = daemon.clone();
        let daemon_thread = thread::spawn(move|| {
            Daemon::run(cloned_daemon);
        });
        let one_sec = Duration::from_millis(1000);
        thread::sleep(one_sec);
        Manager::add_account("caroline.gz", "GLaDOs is a bad robot", true);
        Manager::add_account("Cave Johnson", "Chell is my daughter", false);
        let storage = daemon.lock().unwrap().storage.clone();
        let accounts_added = storage.lock().unwrap().accounts_added.clone();
        assert!(accounts_added.len() == 2);
        let caroline = accounts_added.first().unwrap();
        assert!(caroline.get("Account.archivePath").unwrap() == "caroline.gz");
        assert!(caroline.get("Account.type").unwrap() == "RING");
        assert!(caroline.get("Account.archivePassword").unwrap() == "GLaDOs is a bad robot");
        let cave = accounts_added.last().unwrap();
        assert!(cave.get("Account.alias").unwrap() == "Cave Johnson");
        assert!(cave.get("Account.type").unwrap() == "RING");
        assert!(cave.get("Account.archivePassword").unwrap() == "Chell is my daughter");
        daemon.lock().unwrap().stop();
        let _ = daemon_thread.join();
        teardown();
    }

    // NOTE: I don't test handle_signals because it's just about interfacing signals and server.
    // So, tests are in test_server.rs
}