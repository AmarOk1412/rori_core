extern crate core;
extern crate dbus;
extern crate time;

mod mocks;

#[cfg(test)]
mod tests_server {
    use core::rori::account::Account;
    use core::rori::database::Database;
    use core::rori::interaction::Interaction;
    use core::rori::server::Server;
    use core::rori::user::{Device, User};
    use mocks::Daemon;
    use std::fs;
    use std::sync::atomic::Ordering;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;
    use time;

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
    fn test_formatted_account() {
        let server = setup(User::new(), Vec::new());
        let account = server.account;
        let formatted_account = format!("{}", account);
        assert!(formatted_account == format!("[{}]: {} ({}) - Active: {}", account.id, account.ring_id, account.alias, account.enabled));
        teardown();
    }

    #[test]
    fn test_formatted_interaction() {
        let interaction = Interaction {
            author_ring_id: String::from("Tars_id"),
            body: String::from("My joke percentage is at 70%!"),
            datatype: String::from("text/plain"),
            time: time::now()
        };
        let formatted_account = format!("{}", interaction);
        assert!(formatted_account == format!("{} ({}): {}", interaction.author_ring_id, interaction.datatype, interaction.body));
    }

    #[test]
    // Scenario:
    // 1. Ask the server to add some anonymouses
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

    #[test]
    // Scenario:
    // 1. Someone try to get some ring_id from devicenames or usernames
    fn server_get_ring_id() {
        let mut registered = User::new();
        registered.name = String::from("PBody");
        let mut pbody = Device::new(&String::from("PBody_id"));
        pbody.name = String::from("device");
        registered.devices.push(pbody);
        let mut users = Vec::new();
        users.push(registered);
        let mut server = setup(User::new(), users);
        // RORI reserved
        let ring_id = server.get_ring_id(&String::from("RORI"));
        assert!(ring_id == "GLaDOs_ring_id");
        let ring_id = server.get_ring_id(&String::from("Rori"));
        assert!(ring_id == "GLaDOs_ring_id");
        let ring_id = server.get_ring_id(&String::from("ROri"));
        assert!(ring_id == "GLaDOs_ring_id");
        let ring_id = server.get_ring_id(&String::from("rori"));
        assert!(ring_id == "GLaDOs_ring_id");
        // Search user
        let ring_id = server.get_ring_id(&String::from("PBody"));
        assert!(ring_id == "PBody_id");
        // Search device
        let ring_id = server.get_ring_id(&String::from("PBody_device"));
        assert!(ring_id == "PBody_id");
        // Not here
        let ring_id = server.get_ring_id(&String::from("Atlas"));
        assert!(ring_id.len() == 0);
        teardown();
    }

    #[test]
    // Scenario
    // 1. Build the server from previous session with different accounts
    fn server_load_devices() {
        let mut server = setup(User::new(), Vec::new());
        // load_devices
        let mut devices = Vec::new();
        // anonymous user
        devices.push((String::from("GLaDOs"), String::from(""), String::from("")));
        devices.push((String::from("PBOdy"), String::from(""), String::from("")));
        // Create new user
        devices.push((String::from("Alexa"), String::from("Alexa"), String::from("Alexa")));
        devices.push((String::from("Home"), String::from("Home"), String::from("Home")));
        // Add to known
        devices.push((String::from("Tars"), String::from("Alexa"), String::from("Tars")));
        server.load_devices(devices);
        // anonymous should containe 2 devices
        assert!(server.anonymous_user.devices.len() == 2);
        assert!(server.registered_users.len() == 2);
        // Device can be retrieven
        for dev in &server.anonymous_user.devices {
            assert!(dev.ring_id == "GLaDOs" || dev.ring_id == "PBOdy");
        }
        for user in &server.registered_users {
            assert!(user.name == "Alexa" || user.name == "Home");
            for dev in &user.devices {
                if user.name == "Alexa" {
                    assert!((dev.name == "Alexa" && dev.ring_id == "Alexa")
                            || (dev.name == "Tars" && dev.ring_id == "Tars"));
                } else if user.name == "Home" {
                    assert!((dev.name == "Home" && dev.ring_id == "Home"));
                } else {
                    panic!("Incorrect user found");
                }
            }
        }
        teardown();
    }

    #[test]
    // Scenario
    // 1. A new contact never added send an interaction
    fn server_add_unknown_from_interaction() {
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
            time: time::now()
        });
        // Should be in anonymouses
        assert!(server.anonymous_user.devices.len() == 1);
        // Device can be retrieven
        for dev in &server.anonymous_user.devices {
            assert!(dev.ring_id == "Tars_id");
        }
        // Contact should be added daemon side
        let mut idx_signal = 0;
        let hundred_millis = Duration::from_millis(100);
        while idx_signal < 10 {
            let storage = daemon.lock().unwrap().storage.clone();
            let has_new_info = storage.lock().unwrap().new_info.load(Ordering::SeqCst);
            if has_new_info {
                let contacts_added = storage.lock().unwrap().contacts_added.clone();
                assert!(contacts_added.len() == 1);
                for contact in contacts_added {
                    assert!(contact.0 == "GLaDOs_id");
                    assert!(contact.1 == "Tars_id");
                }
                break;
            }
            thread::sleep(hundred_millis);
            idx_signal += 1;
            if idx_signal == 10 {
                panic!("Contact not set!");
            }
        }
        teardown();
        daemon.lock().unwrap().stop();
        let _ = daemon_thread.join();
    }

    #[test]
    // Scenario
    // 1. A contact send /register username
    // 2. Another contact send /register username
    fn server_register() {
        let daemon = Arc::new(Mutex::new(Daemon::new()));
        let cloned_daemon = daemon.clone();
        let daemon_thread = thread::spawn(move|| {
            Daemon::run(cloned_daemon);
        });
        let mut anonymous = User::new();
        anonymous.devices.push(Device::new(&String::from("Tars_id")));
        anonymous.devices.push(Device::new(&String::from("Bad_Tars_id")));
        let mut server = setup(anonymous, Vec::new());
        assert!(server.anonymous_user.devices.len() == 2);
        // Tars_id do a /register
        server.handle_interaction(Interaction {
            author_ring_id: String::from("Tars_id"),
            body: String::from("/register tars"),
            datatype: String::from("rori/command"),
            time: time::now()
        });
        // And bad tars try to to the same thing
        server.handle_interaction(Interaction {
            author_ring_id: String::from("Bad_Tars_id"),
            body: String::from("/register tars"),
            datatype: String::from("rori/command"),
            time: time::now()
        });
        // Bad_Tars_id should still be an anonymous
        assert!(server.anonymous_user.devices.len() == 1);
        assert!(server.anonymous_user.devices.first().unwrap().ring_id == "Bad_Tars_id");
        // Tars_id should be a user now.
        assert!(server.registered_users.len() == 1);
        assert!(server.registered_users.first().unwrap().name == "tars");
        assert!(server.registered_users.first().unwrap().devices.first().unwrap().ring_id == "Tars_id");
        // This should has sent 2 messages
        let mut idx_signal = 0;
        let hundred_millis = Duration::from_millis(100);
        while idx_signal < 10 {
            let storage = daemon.lock().unwrap().storage.clone();
            let has_new_info = storage.lock().unwrap().new_info.load(Ordering::SeqCst);
            if has_new_info {
                let interactions = storage.lock().unwrap().interactions_sent.clone();
                assert!(interactions.len() == 2);
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
    // 1. A User send /add_device devicename
    // 2. A User send /add_device devicename2 where username_devicename2 is already taken
    fn server_register_device() {
        let daemon = Arc::new(Mutex::new(Daemon::new()));
        let cloned_daemon = daemon.clone();
        let daemon_thread = thread::spawn(move|| {
            Daemon::run(cloned_daemon);
        });
        let mut tars = User::new();
        tars.name = String::from("Tars");
        tars.devices.push(Device::new(&String::from("Tars_id")));
        tars.devices.push(Device::new(&String::from("Tars_id2")));
        let mut badtars = User::new();
        badtars.name = String::from("Tars_pc");
        badtars.devices.push(Device::new(&String::from("Tars_pc")));
        let mut users = Vec::new();
        users.push(tars);
        users.push(badtars);
        let mut server = setup(User::new(), users);


        // Tars_id do a /add_device android (should succeed)
        server.handle_interaction(Interaction {
            author_ring_id: String::from("Tars_id"),
            body: String::from("/add_device android"),
            datatype: String::from("rori/command"),
            time: time::now()
        });
        // Tars_id2 do a /add_device pc (should fails because of Tars_pc)
        server.handle_interaction(Interaction {
            author_ring_id: String::from("Tars_id2"),
            body: String::from("/add_device pc"),
            datatype: String::from("rori/command"),
            time: time::now()
        });
        // Tars_id should now be recognized for Tars_android
        let mut confirmed = false;
        for user in &server.registered_users {
            if user.name == "Tars" {
                for device in &user.devices {
                    if device.ring_id == "Tars_id" {
                        assert!(device.name == "android");
                    } else {
                        assert!(device.name.len() == 0);
                    }
                }
                confirmed = true;
            }
        }
        if !confirmed {
            panic!("Tars not found");
        }
        // This should has sent 2 messages
        let mut idx_signal = 0;
        let hundred_millis = Duration::from_millis(100);
        while idx_signal < 10 {
            let storage = daemon.lock().unwrap().storage.clone();
            let has_new_info = storage.lock().unwrap().new_info.load(Ordering::SeqCst);
            if has_new_info {
                let interactions = storage.lock().unwrap().interactions_sent.clone();
                assert!(interactions.len() == 2);
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
    // 1. A User with multiple device send /add_device devicename device2
    // 2. A User send /add_device devicename2 device_from_another_user
    fn server_register_other_device() {
        let daemon = Arc::new(Mutex::new(Daemon::new()));
        let cloned_daemon = daemon.clone();
        let daemon_thread = thread::spawn(move|| {
            Daemon::run(cloned_daemon);
        });
        let mut tars = User::new();
        tars.name = String::from("Tars");
        tars.devices.push(Device::new(&String::from("Tars_id")));
        tars.devices.push(Device::new(&String::from("Tars_id2")));
        let mut badtars = User::new();
        badtars.name = String::from("Atlas");
        badtars.devices.push(Device::new(&String::from("Atlas_id")));
        let mut users = Vec::new();
        users.push(tars);
        users.push(badtars);
        let mut server = setup(User::new(), users);


        // Tars_id do a /add_device pc Tars_id2 (should succeed)
        server.handle_interaction(Interaction {
            author_ring_id: String::from("Tars_id"),
            body: String::from("/add_device pc Tars_id2"),
            datatype: String::from("rori/command"),
            time: time::now()
        });
        // Tars_id2 do a /add_device pc2 Atlas_id (should fails)
        server.handle_interaction(Interaction {
            author_ring_id: String::from("Tars_id"),
            body: String::from("/add_device pc2 Atlas_id"),
            datatype: String::from("rori/command"),
            time: time::now()
        });
        // Tars_id2 should now be recognized for Tars_pc and Atlas_id as nothing
        let mut confirmed_tars = false;
        let mut confirmed_atlas = false;
        for user in &server.registered_users {
            if user.name == "Tars" {
                for device in &user.devices {
                    if device.ring_id == "Tars_id2" {
                        assert!(device.name == "pc");
                    } else {
                        assert!(device.name.len() == 0);
                    }
                }
                confirmed_tars = true;
            } else {
                assert!(user.name == "Atlas");
                assert!(user.devices.first().unwrap().ring_id == "Atlas_id");
                assert!(user.devices.first().unwrap().name.len() == 0);
                confirmed_atlas = true;
            }
        }
        if !confirmed_tars || !confirmed_atlas {
            panic!("Tars not found");
        }
        teardown();
        daemon.lock().unwrap().stop();
        let _ = daemon_thread.join();
    }

    #[test]
    // Scenario
    // 1. A User revoke one device
    // 1. A User revoke its last device
    fn server_remove_device() {
        let daemon = Arc::new(Mutex::new(Daemon::new()));
        let cloned_daemon = daemon.clone();
        let daemon_thread = thread::spawn(move|| {
            Daemon::run(cloned_daemon);
        });
        let mut atlas = User::new();
        atlas.name = String::from("Atlas");
        let mut atlas_device = Device::new(&String::from("Atlas_id1"));
        atlas_device.name = String::from("Device");
        atlas.devices.push(atlas_device);
        let mut atlas_device = Device::new(&String::from("Atlas_id2"));
        atlas_device.name = String::from("Device2");
        atlas.devices.push(atlas_device);
        let mut users = Vec::new();
        users.push(atlas);
        let mut server = setup(User::new(), users);
        assert!(server.anonymous_user.devices.len() == 0);

        // Atlas_id1 do a /rm_devce (should succeed)
        server.handle_interaction(Interaction {
            author_ring_id: String::from("Atlas_id1"),
            body: String::from("/rm_device"),
            datatype: String::from("rori/command"),
            time: time::now()
        });

        assert!(server.anonymous_user.devices.len() == 1);
        assert!(server.anonymous_user.devices.first().unwrap().ring_id == "Atlas_id1");
        assert!(server.registered_users.len() == 1);
        assert!(server.registered_users.first().unwrap().devices.first().unwrap().ring_id == "Atlas_id2");

        // Atlas_id2 do a /rm_devce (should succeed)
        server.handle_interaction(Interaction {
            author_ring_id: String::from("Atlas_id2"),
            body: String::from("/rm_device"),
            datatype: String::from("rori/command"),
            time: time::now()
        });

        assert!(server.anonymous_user.devices.len() == 2);
        assert!(server.registered_users.len() == 0);

        // This should has sent 2 messages
        let mut idx_signal = 0;
        let hundred_millis = Duration::from_millis(100);
        while idx_signal < 10 {
            let storage = daemon.lock().unwrap().storage.clone();
            let has_new_info = storage.lock().unwrap().new_info.load(Ordering::SeqCst);
            if has_new_info {
                let interactions = storage.lock().unwrap().interactions_sent.clone();
                assert!(interactions.len() == 2);
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
    // 1. A User revoke one of its another device
    // 1. A User try to revoke the device of someone else
    fn server_remove_other_device() {
        let daemon = Arc::new(Mutex::new(Daemon::new()));
        let cloned_daemon = daemon.clone();
        let daemon_thread = thread::spawn(move|| {
            Daemon::run(cloned_daemon);
        });
        let mut atlas = User::new();
        atlas.name = String::from("Atlas");
        let mut atlas_device = Device::new(&String::from("Atlas_id1"));
        atlas_device.name = String::from("Device");
        atlas.devices.push(atlas_device);
        let mut atlas_device = Device::new(&String::from("Atlas_id2"));
        atlas_device.name = String::from("Device2");
        atlas.devices.push(atlas_device);
        let mut tars = User::new();
        tars.name = String::from("Tars");
        let mut tars_device = Device::new(&String::from("Tars_id1"));
        tars_device.name = String::from("Device");
        tars.devices.push(tars_device);
        let mut users = Vec::new();
        users.push(atlas);
        users.push(tars);
        let mut server = setup(User::new(), users);
        assert!(server.anonymous_user.devices.len() == 0);

        // Atlas_id1 do a /rm_device Atlas_id2 (should succeed)
        server.handle_interaction(Interaction {
            author_ring_id: String::from("Atlas_id1"),
            body: String::from("/rm_device Atlas_id2"),
            datatype: String::from("rori/command"),
            time: time::now()
        });

        assert!(server.anonymous_user.devices.len() == 1);
        assert!(server.anonymous_user.devices.first().unwrap().ring_id == "Atlas_id2");
        assert!(server.registered_users.len() == 2);
        assert!(server.registered_users.first().unwrap().devices.first().unwrap().ring_id == "Atlas_id1");

        // Atlas_id1 do a /rm_device Tars_id1 (should fails)
        server.handle_interaction(Interaction {
            author_ring_id: String::from("Atlas_id2"),
            body: String::from("/rm_device Tars_id1"),
            datatype: String::from("rori/command"),
            time: time::now()
        });

        assert!(server.anonymous_user.devices.len() == 1);
        assert!(server.registered_users.len() == 2);
        assert!(server.registered_users.last().unwrap().devices.first().unwrap().ring_id == "Tars_id1");

        teardown();
        daemon.lock().unwrap().stop();
        let _ = daemon_thread.join();
    }

    #[test]
    // Scenario
    // 1. A user authorizes another device to link
    // 2. The other device link to this user
    // 3. Another device ask a user to link
    // 4. a user accepts.
    fn server_link_device() {
        let daemon = Arc::new(Mutex::new(Daemon::new()));
        let cloned_daemon = daemon.clone();
        let daemon_thread = thread::spawn(move|| {
            Daemon::run(cloned_daemon);
        });
        let mut server = setup(User::new(), Vec::new());
        server.add_new_anonymous_device(&String::from("Atlas_id1"));
        server.add_new_anonymous_device(&String::from("Atlas_id2"));
        server.add_new_anonymous_device(&String::from("Atlas_id3"));
        server.handle_interaction(Interaction {
            author_ring_id: String::from("Atlas_id1"),
            body: String::from("/register Atlas"),
            datatype: String::from("rori/command"),
            time: time::now()
        });
        assert!(server.anonymous_user.devices.len() == 2);
        assert!(server.registered_users.len() == 1);

        // Atlas_id authorizes Atlas_id2 to link
        server.handle_interaction(Interaction {
            author_ring_id: String::from("Atlas_id1"),
            body: String::from("/link Atlas_id2"),
            datatype: String::from("rori/command"),
            time: time::now()
        });
        assert!(server.anonymous_user.devices.len() == 2);
        assert!(server.registered_users.first().unwrap().devices.len() == 1);
        // Atlas_id2 link to Atlas
        server.handle_interaction(Interaction {
            author_ring_id: String::from("Atlas_id2"),
            body: String::from("/link Atlas"),
            datatype: String::from("rori/command"),
            time: time::now()
        });

        assert!(server.anonymous_user.devices.len() == 1);
        assert!(server.anonymous_user.devices.first().unwrap().ring_id == "Atlas_id3");
        assert!(server.registered_users.first().unwrap().devices.len() == 2);

        // Atlas_id3 asks to be linked to Atlas
        server.handle_interaction(Interaction {
            author_ring_id: String::from("Atlas_id3"),
            body: String::from("/link Atlas"),
            datatype: String::from("rori/command"),
            time: time::now()
        });
        assert!(server.anonymous_user.devices.len() == 1);
        assert!(server.registered_users.first().unwrap().devices.len() == 2);
        // Atlas_id authorizes Atlas_id3 to be linked
        server.handle_interaction(Interaction {
            author_ring_id: String::from("Atlas_id1"),
            body: String::from("/link Atlas_id3"),
            datatype: String::from("rori/command"),
            time: time::now()
        });
        assert!(server.anonymous_user.devices.len() == 0);

        // This should has sent 3 messages
        let mut idx_signal = 0;
        let hundred_millis = Duration::from_millis(100);
        while idx_signal < 10 {
            let storage = daemon.lock().unwrap().storage.clone();
            let has_new_info = storage.lock().unwrap().new_info.load(Ordering::SeqCst);
            if has_new_info {
                let interactions = storage.lock().unwrap().interactions_sent.clone();
                assert!(interactions.len() == 3);
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
    // 1. A user unregister himself with multiple devices
    fn server_unregister() {
        let daemon = Arc::new(Mutex::new(Daemon::new()));
        let cloned_daemon = daemon.clone();
        let daemon_thread = thread::spawn(move|| {
            Daemon::run(cloned_daemon);
        });
        let mut atlas = User::new();
        atlas.name = String::from("Atlas");
        atlas.devices.push(Device::new(&String::from("Atlas_id1")));
        atlas.devices.push(Device::new(&String::from("Atlas_id2")));
        let mut users = Vec::new();
        users.push(atlas);
        let mut server = setup(User::new(), users);
        assert!(server.registered_users.len() == 1);
        assert!(server.registered_users.first().unwrap().devices.len() == 2);

        // Atlas_id1 unregister user.
        server.handle_interaction(Interaction {
            author_ring_id: String::from("Atlas_id1"),
            body: String::from("/unregister"),
            datatype: String::from("rori/command"),
            time: time::now()
        });
        assert!(server.anonymous_user.devices.len() == 2);
        assert!(server.registered_users.len() == 0);

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
}
