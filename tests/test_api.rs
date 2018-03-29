extern crate core;
extern crate dbus;

mod mocks;


#[cfg(test)]
mod tests_api {
    /*use mocks::Daemon;
    use std::sync::{Arc, Mutex};
    use std::{thread, time};

    #[test]
    fn test_api() {
        // NOTE: temp for testing testing :D
        let daemon = Arc::new(Mutex::new(Daemon::new()));
        let cloned_daemon = daemon.clone();
        let daemon_thread = thread::spawn(move|| {
            Daemon::run(cloned_daemon);
        });

        let hundrer_millis = time::Duration::from_millis(100);
        for _ in 0..10 {
            daemon.lock().unwrap().emit_account_changed();
            thread::sleep(hundrer_millis);
        }
        daemon.lock().unwrap().stop();
        let _ = daemon_thread.join();
    }*/
}
