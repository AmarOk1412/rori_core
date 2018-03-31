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
    use std::sync::{Arc, Mutex};
    use std::thread;

    fn setup() {
        let _ = fs::remove_file("rori.db");
        Database::init_db(); // assert this function is correct.
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
    // 1. Retrieve account list
    fn manager_get_account_list() {
        setup();
        let daemon = Arc::new(Mutex::new(Daemon::new()));
        let cloned_daemon = daemon.clone();
        let daemon_thread = thread::spawn(move|| {
            Daemon::run(cloned_daemon);
        });

        let account_list = Manager::get_account_list();
        assert!(account_list.len() == 1);
        let account = account_list.first().unwrap();
        assert!(account.id == "GLaDOs_id");
        assert!(account.ring_id == "GLaDOs_ring_id");
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
}
