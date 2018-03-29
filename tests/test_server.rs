extern crate core;

#[cfg(test)]
mod tests_api {
    use core::rori::account::Account;
    use core::rori::database::Database;
    use core::rori::server::Server;
    use core::rori::user::User;
    use std::fs;

    fn setup(anonymous: User, registered_users: Vec<User>) -> Server {
        let _ = fs::remove_file("rori.db");
        Database::init_db();
        let account = Account {
            id: String::from("GLaDOs_id"),
            ring_id: String::from("GLaDOs_ring_id"),
            alias: String::from("GLaDOs_alias"),
            enabled: true,
        };
        let mut server = Server::new(account);
        server.registered_users = registered_users;
        server.anonymous_user = anonymous;
        server
    }

    fn teardown() {
        let _ = fs::remove_file("rori.db");
    }

    #[test]
    fn server_add_new_anonymous_device() {
        let mut server = setup(User::new(), Vec::new());
        assert!(Database::get_devices().len() == 0);
        assert!(server.anonymous_user.devices.len() == 0);
        // Insert new devices
        let ok = server.add_new_anonymous_device(&String::from("PBody"));
        assert!(ok);
        let ok = server.add_new_anonymous_device(&String::from("Atlas"));
        assert!(ok);
        // Device can be retrieven
        let device = Database::get_device(&String::from("PBody"));
        assert!(device.0 == "PBody");
        assert!(device.1 == "");
        assert!(device.2 == "");
        // Anonymous user should contains 2 devices
        assert!(server.anonymous_user.devices.len() == 2);
        // Retrieve all devices and no more
        for device in &server.anonymous_user.devices {
            if device.ring_id == "PBody" || device.ring_id == "Atlas" {
            } else {
                panic!("Unknown user");
            }
        }
        // Reinsert devices should do nothing
        let ok = server.add_new_anonymous_device(&String::from("PBody"));
        assert!(!ok);
        teardown();
    }
}
