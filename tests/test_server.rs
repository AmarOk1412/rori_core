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
    use std::collections::HashMap;
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
            ring_id: String::from("GLaDOs_hash"),
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
        let mut metadatas: HashMap<String, String> = HashMap::new();
        metadatas.insert(String::from("foo"), String::from("bar"));
        let interaction = Interaction {
            device_author: Device {
                id: 0,
                name: String::new(),
                ring_id: String::from("Tars_id"),
                is_bridge: false
            },
            body: String::from("My joke percentage is at 70%!"),
            datatype: String::from("text/plain"),
            time: time::now(),
            metadatas: metadatas,
        };
        let formatted_account = format!("{}", interaction);
        assert!(formatted_account == format!("{} ({};{:?}): {}", interaction.device_author, interaction.datatype, interaction.metadatas, interaction.body));
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
        let device = Database::get_device(&String::from("PBody"), &String::new());
        assert!(device.1 == "PBody");
        assert!(device.2 == "");
        assert!(device.3 == "");
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
    // 1. Ask the server to add an anonymous and add_types
    fn server_add_types() {
        let daemon = Arc::new(Mutex::new(Daemon::new()));
        let cloned_daemon = daemon.clone();
        let daemon_thread = thread::spawn(move|| {
            Daemon::run(cloned_daemon);
        });
        let mut anonymous = User::new();
        anonymous.devices.push(Device::new(&0, &String::from("Tars_id")));
        let mut server = setup(anonymous, Vec::new());
        assert!(server.anonymous_user.devices.len() == 1);
        let did = Database::insert_new_device(&String::from("Tars_id"), &String::new(), &String::new(), false);
        let did = did.ok().unwrap();

        // Tars_id do a /add_types without datatypes
        server.handle_interaction(Interaction {
            device_author: Device {
                id: 0,
                name: String::new(),
                ring_id: String::from("Tars_id"),
                is_bridge: false
            },
            body: String::from("/add_types"),
            datatype: String::from("rori/command"),
            time: time::now(),
            metadatas: HashMap::new(),
        });
        let dt = Database::get_datatypes(&did.clone());
        assert!(dt.len() == 0);

        // Tars_id do a /add_types with 2 datatypes
        server.handle_interaction(Interaction {
            device_author: Device {
                id: 0,
                name: String::new(),
                ring_id: String::from("Tars_id"),
                is_bridge: false
            },
            body: String::from("/add_types music command"),
            datatype: String::from("rori/command"),
            time: time::now(),
            metadatas: HashMap::new(),
        });
        let dt = Database::get_datatypes(&did.clone());
        assert!(dt.len() == 2);
        assert!(dt.first().unwrap() == "music");
        assert!(dt.last().unwrap() == "command");

        // Tars_id do a /add_types with 1 already set and 1 not set
        server.handle_interaction(Interaction {
            device_author: Device {
                id: 0,
                name: String::new(),
                ring_id: String::from("Tars_id"),
                is_bridge: false
            },
            body: String::from("/add_types music other"),
            datatype: String::from("rori/command"),
            time: time::now(),
            metadatas: HashMap::new(),
        });
        let dt = Database::get_datatypes(&did);
        assert!(dt.len() == 3);
        assert!(dt.last().unwrap() == "other");


        teardown();
        daemon.lock().unwrap().stop();
        let _ = daemon_thread.join();
    }

    #[test]
    // Scenario:
    // 1. Ask the server to add an anonymous and add_types then rm_types
    fn server_rm_types() {
        let daemon = Arc::new(Mutex::new(Daemon::new()));
        let cloned_daemon = daemon.clone();
        let daemon_thread = thread::spawn(move|| {
            Daemon::run(cloned_daemon);
        });
        let mut anonymous = User::new();
        anonymous.devices.push(Device::new(&0, &String::from("Tars_id")));
        let mut server = setup(anonymous, Vec::new());
        assert!(server.anonymous_user.devices.len() == 1);
        let _ = Database::insert_new_device(&String::from("Tars_id"), &String::new(), &String::new(), false);

        // Tars_id do a /add_types with 3 datatypes
        server.handle_interaction(Interaction {
            device_author: Device {
                id: 0,
                name: String::new(),
                ring_id: String::from("Tars_id"),
                is_bridge: false
            },
            body: String::from("/add_types music command other"),
            datatype: String::from("rori/command"),
            time: time::now(),
            metadatas: HashMap::new(),
        });
        let dt = Database::get_datatypes(&1);
        assert!(dt.len() == 3);

        // Tars_id do a /rm_types with nothing
        server.handle_interaction(Interaction {
            device_author: Device {
                id: 0,
                name: String::new(),
                ring_id: String::from("Tars_id"),
                is_bridge: false
            },
            body: String::from("/rm_types"),
            datatype: String::from("rori/command"),
            time: time::now(),
            metadatas: HashMap::new(),
        });
        let dt = Database::get_datatypes(&1);
        assert!(dt.len() == 3);

        // Tars_id do a /rm_types with something not in types
        server.handle_interaction(Interaction {
            device_author: Device {
                id: 0,
                name: String::new(),
                ring_id: String::from("Tars_id"),
                is_bridge: false
            },
            body: String::from("/rm_types nothing"),
            datatype: String::from("rori/command"),
            time: time::now(),
            metadatas: HashMap::new(),
        });
        let dt = Database::get_datatypes(&1);
        assert!(dt.len() == 3);

        // Tars_id do a /rm_types with something not in types and 2 in types
        server.handle_interaction(Interaction {
            device_author: Device {
                id: 0,
                name: String::new(),
                ring_id: String::from("Tars_id"),
                is_bridge: false
            },
            body: String::from("/rm_types nothing music command"),
            datatype: String::from("rori/command"),
            time: time::now(),
            metadatas: HashMap::new(),
        });
        let dt = Database::get_datatypes(&1);
        assert!(dt.len() == 1);
        assert!(dt.first().unwrap() == "other");

        // Tars_id do a /rm_types with last one
        server.handle_interaction(Interaction {
            device_author: Device {
                id: 0,
                name: String::new(),
                ring_id: String::from("Tars_id"),
                is_bridge: false
            },
            body: String::from("/rm_types other"),
            datatype: String::from("rori/command"),
            time: time::now(),
            metadatas: HashMap::new(),
        });
        let dt = Database::get_datatypes(&1);
        assert!(dt.len() == 0);

        teardown();
        daemon.lock().unwrap().stop();
        let _ = daemon_thread.join();
    }

    #[test]
    // Scenario:
    // 1. Ask the server to add an anonymous and set_types
    fn server_set_types() {
        let daemon = Arc::new(Mutex::new(Daemon::new()));
        let cloned_daemon = daemon.clone();
        let daemon_thread = thread::spawn(move|| {
            Daemon::run(cloned_daemon);
        });
        let mut anonymous = User::new();
        anonymous.devices.push(Device::new(&0, &String::from("Tars_id")));
        let mut server = setup(anonymous, Vec::new());
        assert!(server.anonymous_user.devices.len() == 1);
        let _ = Database::insert_new_device(&String::from("Tars_id"), &String::new(), &String::new(), false);

        // Tars_id do a /add_types with 3 datatypes
        server.handle_interaction(Interaction {
            device_author: Device {
                id: 0,
                name: String::new(),
                ring_id: String::from("Tars_id"),
                is_bridge: false
            },
            body: String::from("/add_types music command other"),
            datatype: String::from("rori/command"),
            time: time::now(),
            metadatas: HashMap::new(),
        });
        let dt = Database::get_datatypes(&1);
        assert!(dt.len() == 3);

        // Tars_id do a /set_types with two types
        server.handle_interaction(Interaction {
            device_author: Device {
                id: 0,
                name: String::new(),
                ring_id: String::from("Tars_id"),
                is_bridge: false
            },
            body: String::from("/set_types type1 type2"),
            datatype: String::from("rori/command"),
            time: time::now(),
            metadatas: HashMap::new(),
        });
        let dt = Database::get_datatypes(&1);
        assert!(dt.len() == 2);
        assert!(dt.first().unwrap() == "type1");
        assert!(dt.last().unwrap() == "type2");

        // Tars_id do a /set_types with nothing
        server.handle_interaction(Interaction {
            device_author: Device {
                id: 0,
                name: String::new(),
                ring_id: String::from("Tars_id"),
                is_bridge: false
            },
            body: String::from("/set_types "),
            datatype: String::from("rori/command"),
            time: time::now(),
            metadatas: HashMap::new(),
        });
        let dt = Database::get_datatypes(&1);
        assert!(dt.len() == 0);

        teardown();
        daemon.lock().unwrap().stop();
        let _ = daemon_thread.join();
    }


    #[test]
    // Scenario:
    // 1. Someone try to get some hash from devicenames or usernames
    fn server_get_hash() {
        let mut registered = User::new();
        registered.name = String::from("PBody");
        let mut pbody = Device::new(&0, &String::from("PBody_id"));
        pbody.name = String::from("device");
        registered.devices.push(pbody);
        let mut users = Vec::new();
        users.push(registered);
        let mut server = setup(User::new(), users);
        // RORI reserved
        let hash = server.get_hash(&String::from("RORI"));
        assert!(hash == "GLaDOs_hash");
        let hash = server.get_hash(&String::from("Rori"));
        assert!(hash == "GLaDOs_hash");
        let hash = server.get_hash(&String::from("ROri"));
        assert!(hash == "GLaDOs_hash");
        let hash = server.get_hash(&String::from("rori"));
        assert!(hash == "GLaDOs_hash");
        // Search user
        let hash = server.get_hash(&String::from("PBody"));
        assert!(hash == "PBody_id");
        // Search device
        let hash = server.get_hash(&String::from("PBody_device"));
        assert!(hash == "PBody_id");
        // Not here
        let hash = server.get_hash(&String::from("Atlas"));
        assert!(hash.len() == 0);
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
        devices.push((0, String::from("GLaDOs"), String::from(""), String::from(""), false));
        devices.push((1, String::from("PBOdy"), String::from(""), String::from(""), false));
        // Create new user
        devices.push((2, String::from("Alexa"), String::from("Alexa"), String::from("Alexa"), false));
        devices.push((3, String::from("Home"), String::from("Home"), String::from("Home"), false));
        // Add to known
        devices.push((4, String::from("Tars"), String::from("Alexa"), String::from("Tars"), false));
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
            device_author: Device {
                id: 0,
                name: String::new(),
                ring_id: String::from("Tars_id"),
                is_bridge: false
            },
            body: String::from("My joke percentage is at 70%!"),
            datatype: String::from("text/plain"),
            time: time::now(),
            metadatas: HashMap::new()
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
        anonymous.devices.push(Device::new(&0, &String::from("Tars_id")));
        anonymous.devices.push(Device::new(&1, &String::from("Bad_Tars_id")));
        let mut server = setup(anonymous, Vec::new());
        assert!(server.anonymous_user.devices.len() == 2);
        // Tars_id do a /register
        server.handle_interaction(Interaction {
            device_author: Device {
                id: 0,
                name: String::new(),
                ring_id: String::from("Tars_id"),
                is_bridge: false
            },
            body: String::from("/register tars"),
            datatype: String::from("rori/command"),
            time: time::now(),
            metadatas: HashMap::new()
        });
        // And bad tars try to to the same thing
        server.handle_interaction(Interaction {
            device_author: Device {
                id: 1,
                name: String::new(),
                ring_id: String::from("Bad_Tars_id"),
                is_bridge: false
            },
            body: String::from("/register tars"),
            datatype: String::from("rori/command"),
            time: time::now(),
            metadatas: HashMap::new()
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
    // 1. A contact send /register without username
    fn server_register_without_username() {
        let daemon = Arc::new(Mutex::new(Daemon::new()));
        let cloned_daemon = daemon.clone();
        let daemon_thread = thread::spawn(move|| {
            Daemon::run(cloned_daemon);
        });
        let mut anonymous = User::new();
        anonymous.devices.push(Device::new(&0, &String::from("Tars_id")));
        let mut server = setup(anonymous, Vec::new());
        assert!(server.anonymous_user.devices.len() == 1);
        // Tars_id do a /register
        server.handle_interaction(Interaction {
            device_author: Device {
                id: 0,
                name: String::new(),
                ring_id: String::from("Tars_id"),
                is_bridge: false
            },
            body: String::from("/register"),
            datatype: String::from("rori/command"),
            time: time::now(),
            metadatas: HashMap::new(),
        });
        // Bad_Tars_id should still be an anonymous
        assert!(server.anonymous_user.devices.len() == 1);
        assert!(server.anonymous_user.devices.first().unwrap().ring_id == "Tars_id");
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
        tars.devices.push(Device::new(&1, &String::from("Tars_id")));
        tars.devices.push(Device::new(&2, &String::from("Tars_id2")));
        let mut badtars = User::new();
        badtars.name = String::from("Tars_pc");
        badtars.devices.push(Device::new(&3, &String::from("Tars_pc")));
        let mut users = Vec::new();
        users.push(tars);
        users.push(badtars);
        let mut server = setup(User::new(), users);
        let _ = Database::insert_new_device(&String::from("Tars_id"), &String::from("Tars"), &String::from(""), false);
        let _ = Database::insert_new_device(&String::from("Tars_id2"), &String::from("Tars"), &String::from(""), false);

        // Tars_id do a /add_device android (should succeed)
        server.handle_interaction(Interaction {
            device_author: Device {
                id: 1,
                name: String::from("Tars"),
                ring_id: String::from("Tars_id"),
                is_bridge: false
            },
            body: String::from("/add_device android"),
            datatype: String::from("rori/command"),
            time: time::now(),
            metadatas: HashMap::new(),
        });
        // Tars_id2 do a /add_device pc (should fails because of Tars_pc)
        server.handle_interaction(Interaction {
            device_author: Device {
                id: 2,
                name: String::from("Tars"),
                ring_id: String::from("Tars_id2"),
                is_bridge: false
            },
            body: String::from("/add_device pc"),
            datatype: String::from("rori/command"),
            time: time::now(),
            metadatas: HashMap::new(),
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
    // 1. A User send /add_device devicename from another user
    fn server_register_bad_device() {
        let daemon = Arc::new(Mutex::new(Daemon::new()));
        let cloned_daemon = daemon.clone();
        let daemon_thread = thread::spawn(move|| {
            Daemon::run(cloned_daemon);
        });
        let mut tars = User::new();
        tars.name = String::from("Tars");
        tars.devices.push(Device::new(&1, &String::from("Tars_id")));
        tars.devices.push(Device::new(&2, &String::from("Tars_id2")));
        let mut badtars = User::new();
        badtars.name = String::from("Tars_pc");
        badtars.devices.push(Device::new(&3, &String::from("Tars_pc")));
        let mut users = Vec::new();
        users.push(tars);
        users.push(badtars);
        let mut server = setup(User::new(), users);
        let _ = Database::insert_new_device(&String::from("Tars_id"), &String::from("Tars"), &String::from(""), false);
        let _ = Database::insert_new_device(&String::from("Tars_id1"), &String::from("Tars"), &String::from(""), false);
        let _ = Database::insert_new_device(&String::from("Tars_pc"), &String::from("Tars_pc"), &String::from(""), false);

        // Tars_pc do a /add_device pc of Tars_id (should fails because of it's someone else device)
        server.handle_interaction(Interaction {
            device_author: Device {
                id: 3,
                name: String::new(),
                ring_id: String::from("Tars_pc"),
                is_bridge: false
            },
            body: String::from("/add_device pc Tars_id"),
            datatype: String::from("rori/command"),
            time: time::now(),
            metadatas: HashMap::new(),
        });
        // Tars_id should now be recognized for Tars_android
        let mut confirmed = false;
        for user in &server.registered_users {
            if user.name == "Tars" {
                for device in &user.devices {
                    if device.ring_id == "Tars_id" {
                        assert!(device.name.len() == 0);
                        confirmed = true;
                    }
                }
            }
        }
        if !confirmed {
            panic!("Tars_id not found");
        }
        // This should has sent some messages
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
    // 1. A User send /add_device without devicename
    fn server_register_device_invalid() {
        let daemon = Arc::new(Mutex::new(Daemon::new()));
        let cloned_daemon = daemon.clone();
        let daemon_thread = thread::spawn(move|| {
            Daemon::run(cloned_daemon);
        });
        let mut tars = User::new();
        tars.name = String::from("Tars");
        tars.devices.push(Device::new(&0, &String::from("Tars_id")));
        let mut users = Vec::new();
        users.push(tars);
        let mut server = setup(User::new(), users);

        // Tars_id do a /add_device (should fails)
        server.handle_interaction(Interaction {
            device_author: Device {
                id: 0,
                name: String::new(),
                ring_id: String::from("Tars_id"),
                is_bridge: false
            },
            body: String::from("/add_device"),
            datatype: String::from("rori/command"),
            time: time::now(),
            metadatas: HashMap::new(),
        });
        // Tars_id should now be recognized for Tars_android
        let mut confirmed = false;
        for user in &server.registered_users {
            if user.name == "Tars" {
                for device in &user.devices {
                    if device.ring_id == "Tars_id" {
                        assert!(device.name.len() == 0);
                    }
                }
                confirmed = true;
            }
        }
        if !confirmed {
            panic!("Tars not found");
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
        tars.devices.push(Device::new(&1, &String::from("Tars_id")));
        tars.devices.push(Device::new(&2, &String::from("Tars_id2")));
        let mut badtars = User::new();
        badtars.name = String::from("Atlas");
        badtars.devices.push(Device::new(&3, &String::from("Atlas_id")));
        let mut users = Vec::new();
        users.push(tars);
        users.push(badtars);
        let mut server = setup(User::new(), users);
        let _ = Database::insert_new_device(&String::from("Tars_id"), &String::from("Tars"), &String::from(""), false);
        let _ = Database::insert_new_device(&String::from("Tars_id2"), &String::from("Tars"), &String::from(""), false);
        let _ = Database::insert_new_device(&String::from("Atlas_id"), &String::from("Atlas"), &String::from(""), false);

        // Tars_id do a /add_device pc Tars_id2 (should succeed)
        server.handle_interaction(Interaction {
            device_author: Device {
                id: 1,
                name: String::new(),
                ring_id: String::from("Tars_id"),
                is_bridge: false
            },
            body: String::from("/add_device pc Tars_id2"),
            datatype: String::from("rori/command"),
            time: time::now(),
            metadatas: HashMap::new()
        });
        // Tars_id2 do a /add_device pc2 Atlas_id (should fails)
        server.handle_interaction(Interaction {
            device_author: Device {
                id: 1,
                name: String::new(),
                ring_id: String::from("Tars_id"),
                is_bridge: false
            },
            body: String::from("/add_device pc2 Atlas_id"),
            datatype: String::from("rori/command"),
            time: time::now(),
            metadatas: HashMap::new()
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
        let mut atlas_device = Device::new(&1, &String::from("Atlas_id1"));
        atlas_device.name = String::from("Device");
        atlas.devices.push(atlas_device);
        let mut atlas_device = Device::new(&2, &String::from("Atlas_id2"));
        atlas_device.name = String::from("Device2");
        atlas.devices.push(atlas_device);
        let mut users = Vec::new();
        users.push(atlas);
        let mut server = setup(User::new(), users);
        let _ = Database::insert_new_device(&String::from("Atlas_id1"), &String::from("Atlas"), &String::from("Device"), false);
        let _ = Database::insert_new_device(&String::from("Atlas_id2"), &String::from("Atlas"), &String::from("Device2"), false);
        assert!(server.anonymous_user.devices.len() == 0);

        // Atlas_id1 do a /rm_devce (should succeed)
        server.handle_interaction(Interaction {
            device_author: Device {
                id: 1,
                name: String::from("Device"),
                ring_id: String::from("Atlas_id1"),
                is_bridge: false
            },
            body: String::from("/rm_device"),
            datatype: String::from("rori/command"),
            time: time::now(),
            metadatas: HashMap::new()
        });

        assert!(server.anonymous_user.devices.len() == 1);
        assert!(server.anonymous_user.devices.first().unwrap().ring_id == "Atlas_id1");
        assert!(server.registered_users.len() == 1);
        assert!(server.registered_users.first().unwrap().devices.first().unwrap().ring_id == "Atlas_id2");

        // Atlas_id2 do a /rm_devce (should succeed)
        server.handle_interaction(Interaction {
            device_author: Device {
                id: 2,
                name: String::from("Device"),
                ring_id: String::from("Atlas_id2"),
                is_bridge: false
            },
            body: String::from("/rm_device"),
            datatype: String::from("rori/command"),
            time: time::now(),
            metadatas: HashMap::new()
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
    // 2. A User try to revoke the device of someone else
    fn server_remove_other_device() {
        let daemon = Arc::new(Mutex::new(Daemon::new()));
        let cloned_daemon = daemon.clone();
        let daemon_thread = thread::spawn(move|| {
            Daemon::run(cloned_daemon);
        });
        let mut atlas = User::new();
        atlas.name = String::from("Atlas");
        let mut atlas_device = Device::new(&1, &String::from("Atlas_id1"));
        atlas_device.name = String::from("Device");
        atlas.devices.push(atlas_device);
        let mut atlas_device = Device::new(&2, &String::from("Atlas_id2"));
        atlas_device.name = String::from("Device2");
        atlas.devices.push(atlas_device);
        let mut tars = User::new();
        tars.name = String::from("Tars");
        let mut tars_device = Device::new(&3, &String::from("Tars_id1"));
        tars_device.name = String::from("Device");
        tars.devices.push(tars_device);
        let mut tars_device = Device::new(&4, &String::from("Tars_id2"));
        tars_device.name = String::from("Device2");
        tars.devices.push(tars_device);
        let mut users = Vec::new();
        users.push(atlas);
        users.push(tars);
        let mut server = setup(User::new(), users);
        let _ = Database::insert_new_device(&String::from("Atlas_id1"), &String::from("Atlas"), &String::from(""), false);
        let _ = Database::insert_new_device(&String::from("Atlas_id2"), &String::from("Atlas"), &String::from(""), false);
        let _ = Database::insert_new_device(&String::from("Tars_id1"), &String::from("Tars"), &String::from(""), false);
        let _ = Database::insert_new_device(&String::from("Tars_id2"), &String::from("Tars"), &String::from(""), false);

        assert!(server.anonymous_user.devices.len() == 0);

        // Atlas_id1 do a /rm_device Atlas_id2 (should succeed)
        server.handle_interaction(Interaction {
            device_author: Device {
                id: 4,
                name: String::new(),
                ring_id: String::from("Tars_id2"),
                is_bridge: false
            },
            body: String::from("/rm_device Tars_id1"),
            datatype: String::from("rori/command"),
            time: time::now(),
            metadatas: HashMap::new(),
        });

        assert!(server.anonymous_user.devices.len() == 1);
        assert!(server.anonymous_user.devices.first().unwrap().ring_id == "Tars_id1");
        assert!(server.registered_users.len() == 2);
        assert!(server.registered_users.last().unwrap().devices.first().unwrap().ring_id == "Tars_id2");

        teardown();
        daemon.lock().unwrap().stop();
        let _ = daemon_thread.join();
    }

    #[test]
    // Scenario
    // 1. A User try to revoke the device of nobody
    fn server_remove_invalid_device() {
        let daemon = Arc::new(Mutex::new(Daemon::new()));
        let cloned_daemon = daemon.clone();
        let daemon_thread = thread::spawn(move|| {
            Daemon::run(cloned_daemon);
        });
        let mut atlas = User::new();
        atlas.name = String::from("Atlas");
        let mut atlas_device = Device::new(&1, &String::from("Atlas_id1"));
        atlas_device.name = String::from("Device");
        atlas.devices.push(atlas_device);
        let mut atlas_device = Device::new(&2, &String::from("Atlas_id2"));
        atlas_device.name = String::from("Device2");
        atlas.devices.push(atlas_device);
        let mut tars = User::new();
        tars.name = String::from("Tars");
        let mut tars_device = Device::new(&3, &String::from("Tars_id1"));
        tars_device.name = String::from("Device");
        tars.devices.push(tars_device);
        let mut tars_device = Device::new(&4, &String::from("Tars_id2"));
        tars_device.name = String::from("Device2");
        tars.devices.push(tars_device);
        let mut users = Vec::new();
        users.push(atlas);
        users.push(tars);
        let mut server = setup(User::new(), users);
        assert!(server.anonymous_user.devices.len() == 0);

        // Atlas_id1 do a /rm_device Atlas_id2 (should succeed)
        server.handle_interaction(Interaction {
            device_author: Device {
                id: 4,
                name: String::new(),
                ring_id: String::from("Tars_id2"),
                is_bridge: false
            },
            body: String::from("/rm_device randomId"),
            datatype: String::from("rori/command"),
            time: time::now(),
            metadatas: HashMap::new(),
        });

        assert!(server.registered_users.len() == 2);
        assert!(server.registered_users.last().unwrap().devices.len() == 2);

        teardown();
        daemon.lock().unwrap().stop();
        let _ = daemon_thread.join();
    }


    #[test]
    // Scenario
    // 1. A User send /add_device devicename from another user
    fn server_register_device_someone_else() {
        let daemon = Arc::new(Mutex::new(Daemon::new()));
        let cloned_daemon = daemon.clone();
        let daemon_thread = thread::spawn(move|| {
            Daemon::run(cloned_daemon);
        });
        let mut tars = User::new();
        tars.name = String::from("Tars");
        tars.devices.push(Device::new(&1, &String::from("Tars_id")));
        let mut tars_device = Device::new(&2, &String::from("Tars_id1"));
        tars_device.name = String::from("Device");
        tars.devices.push(tars_device);
        let mut badtars = User::new();
        badtars.name = String::from("Tars_pc");
        badtars.devices.push(Device::new(&3, &String::from("Tars_pc")));
        let mut users = Vec::new();
        users.push(tars);
        users.push(badtars);
        let mut server = setup(User::new(), users);
        let _ = Database::insert_new_device(&String::from("Tars_id"), &String::from("Tars"), &String::from(""), false);
        let _ = Database::insert_new_device(&String::from("Tars_id1"), &String::from("Tars"), &String::from("Device"), false);
        let _ = Database::insert_new_device(&String::from("Tars_pc"), &String::from("Tars_pc"), &String::from(""), false);

        // Tars_pc do a /add_device pc of Tars_id (should fails because of it's someone else device)
        server.handle_interaction(Interaction {
            device_author: Device {
                id: 3,
                name: String::new(),
                ring_id: String::from("Tars_pc"),
                is_bridge: false
            },
            body: String::from("/rm_device Tars_id1"),
            datatype: String::from("rori/command"),
            time: time::now(),
            metadatas: HashMap::new(),
        });
        // Tars_id should now be recognized for Tars_android
        let mut confirmed = false;
        for user in &server.registered_users {
            if user.name == "Tars" {
                for device in &user.devices {
                    if device.ring_id == "Tars_id1" {
                        assert!(device.name == "Device");
                        confirmed = true;
                    }
                }
            }
        }
        if !confirmed {
            panic!("Tars_id1 not found");
        }
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
            device_author: Device {
                id: 1,
                name: String::new(),
                ring_id: String::from("Atlas_id1"),
                is_bridge: false
            },
            body: String::from("/register Atlas"),
            datatype: String::from("rori/command"),
            time: time::now(),
            metadatas: HashMap::new(),
        });
        assert!(server.anonymous_user.devices.len() == 2);
        assert!(server.registered_users.len() == 1);

        // Atlas_id authorizes Atlas_id2 to link
        server.handle_interaction(Interaction {
            device_author: Device {
                id: 1,
                name: String::new(),
                ring_id: String::from("Atlas_id1"),
                is_bridge: false
            },
            body: String::from("/link Atlas_id2"),
            datatype: String::from("rori/command"),
            time: time::now(),
            metadatas: HashMap::new(),
        });
        assert!(server.anonymous_user.devices.len() == 2);
        assert!(server.registered_users.first().unwrap().devices.len() == 1);
        // Atlas_id2 link to Atlas
        server.handle_interaction(Interaction {
            device_author: Device {
                id: 2,
                name: String::new(),
                ring_id: String::from("Atlas_id2"),
                is_bridge: false
            },
            body: String::from("/link Atlas"),
            datatype: String::from("rori/command"),
            time: time::now(),
            metadatas: HashMap::new(),
        });

        assert!(server.anonymous_user.devices.len() == 1);
        assert!(server.anonymous_user.devices.first().unwrap().ring_id == "Atlas_id3");
        assert!(server.registered_users.first().unwrap().devices.len() == 2);

        // Atlas_id3 asks to be linked to Atlas
        server.handle_interaction(Interaction {
            device_author: Device {
                id: 3,
                name: String::new(),
                ring_id: String::from("Atlas_id3"),
                is_bridge: false
            },
            body: String::from("/link Atlas"),
            datatype: String::from("rori/command"),
            time: time::now(),
            metadatas: HashMap::new(),
        });
        assert!(server.anonymous_user.devices.len() == 1);
        assert!(server.registered_users.first().unwrap().devices.len() == 2);
        // Atlas_id authorizes Atlas_id3 to be linked
        server.handle_interaction(Interaction {
            device_author: Device {
                id: 1,
                name: String::new(),
                ring_id: String::from("Atlas_id1"),
                is_bridge: false
            },
            body: String::from("/link Atlas_id3"),
            datatype: String::from("rori/command"),
            time: time::now(),
            metadatas: HashMap::new(),
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
    // 1. A user send /link without any argument
    fn server_link_device_invalid() {
        let daemon = Arc::new(Mutex::new(Daemon::new()));
        let cloned_daemon = daemon.clone();
        let daemon_thread = thread::spawn(move|| {
            Daemon::run(cloned_daemon);
        });
        let mut server = setup(User::new(), Vec::new());
        server.add_new_anonymous_device(&String::from("Atlas_id1"));
        server.handle_interaction(Interaction {
            device_author: Device {
                id: 0,
                name: String::new(),
                ring_id: String::from("Atlas_id1"),
                is_bridge: false
            },
            body: String::from("/register Atlas"),
            datatype: String::from("rori/command"),
            time: time::now(),
            metadatas: HashMap::new(),
        });
        assert!(server.registered_users.len() == 1);

        // Atlas_id authorizes nobody
        server.handle_interaction(Interaction {
            device_author: Device {
                id: 0,
                name: String::new(),
                ring_id: String::from("Atlas_id1"),
                is_bridge: false
            },
            body: String::from("/link"),
            datatype: String::from("rori/command"),
            time: time::now(),
            metadatas: HashMap::new(),
        });
        assert!(server.registered_users.first().unwrap().devices.len() == 1);

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
        let mut eve = User::new();
        eve.name = String::from("Eve");
        eve.devices.push(Device::new(&1, &String::from("Eve_id1")));
        let mut atlas = User::new();
        atlas.name = String::from("Atlas");
        atlas.devices.push(Device::new(&2, &String::from("Atlas_id1")));
        atlas.devices.push(Device::new(&3, &String::from("Atlas_id2")));
        let mut weasley = User::new();
        weasley.name = String::from("Weasley");
        weasley.devices.push(Device::new(&4, &String::from("Weasley_id1")));
        let mut users = Vec::new();
        users.push(eve);
        users.push(atlas);
        users.push(weasley);
        let mut server = setup(User::new(), users);
        let _ = Database::insert_new_device(&String::from("Eve_id1"), &String::from("Eve"), &String::from(""), false);
        let _ = Database::insert_new_device(&String::from("Atlas_id1"), &String::from("Atlas"), &String::from(""), false);
        let _ = Database::insert_new_device(&String::from("Atlas_id2"), &String::from("Atlas"), &String::from(""), false);
        let _ = Database::insert_new_device(&String::from("Weasley_id1"), &String::from("Weasley"), &String::from(""), false);
        assert!(server.registered_users.len() == 3);

        // Atlas_id1 unregister user.
        server.handle_interaction(Interaction {
            device_author: Device {
                id: 2,
                name: String::new(),
                ring_id: String::from("Atlas_id1"),
                is_bridge: false
            },
            body: String::from("/unregister"),
            datatype: String::from("rori/command"),
            metadatas: HashMap::new(),
            time: time::now(),
        });
        assert!(server.anonymous_user.devices.len() == 2);
        assert!(server.registered_users.len() == 2);

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
    // 1. A bridge is sending a fake sub_author
    fn server_bridge_send_fake_subauthor() {
        let daemon = Arc::new(Mutex::new(Daemon::new()));
        let cloned_daemon = daemon.clone();
        let daemon_thread = thread::spawn(move|| {
            Daemon::run(cloned_daemon);
        });
        let mut server = setup(User::new(), Vec::new());
        server.add_new_anonymous_device(&String::from("Atlas_id1"));

        // Add a bridge with one user
        server.handle_interaction(Interaction {
            device_author: Device {
                id: 1,
                name: String::new(),
                ring_id: String::from("Atlas_id1"),
                is_bridge: true
            },
            body: String::from("/bridgify"),
            datatype: String::from("rori/command"),
            time: time::now(),
            metadatas: HashMap::new(),
        });
        server.handle_interaction(Interaction {
            device_author: Device {
                id: 1,
                name: String::new(),
                ring_id: String::from("Atlas_id1"),
                is_bridge: true
            },
            body: String::from("/register Atlas"),
            datatype: String::from("rori/command"),
            time: time::now(),
            metadatas: HashMap::new(),
        });
        assert!(server.registered_users.len() == 1);

        // Now try to unregister Tars. Should fail
        let mut metadatas: HashMap<String, String> = HashMap::new();
        metadatas.insert(String::from("sa"), String::from("Tars"));
        server.handle_interaction(Interaction {
            device_author: Device {
                id: 1,
                name: String::new(),
                ring_id: String::from("Atlas_id1"),
                is_bridge: true
            },
            body: String::from("/unregister"),
            datatype: String::from("rori/command"),
            time: time::now(),
            metadatas: metadatas,
        });
        // Still one registered user
        assert!(server.registered_users.len() == 1);

        teardown();
        daemon.lock().unwrap().stop();
        let _ = daemon_thread.join();
    }

    #[test]
    // Scenario
    // 1. A user unregister himself but no name found
    fn server_unregister_anonymous() {
        let daemon = Arc::new(Mutex::new(Daemon::new()));
        let cloned_daemon = daemon.clone();
        let daemon_thread = thread::spawn(move|| {
            Daemon::run(cloned_daemon);
        });
        let mut anonymous = User::new();
        anonymous.name = String::new();
        anonymous.devices.push(Device::new(&0, &String::from("Atlas_id1")));
        anonymous.devices.push(Device::new(&1, &String::from("Atlas_id2")));
        let users = Vec::new();
        let mut server = setup(anonymous, users);
        assert!(server.anonymous_user.devices.len() == 2);
        assert!(server.registered_users.len() == 0);

        // Atlas_id1 unregister user.
        server.handle_interaction(Interaction {
            device_author: Device {
                id: 0,
                name: String::new(),
                ring_id: String::from("Atlas_id1"),
                is_bridge: false
            },
            body: String::from("/unregister"),
            datatype: String::from("rori/command"),
            time: time::now(),
            metadatas: HashMap::new(),
        });
        // Should change nothing
        assert!(server.anonymous_user.devices.len() == 2);
        assert!(server.registered_users.len() == 0);

        // This should has sent 1 message
        let mut idx_signal = 0;
        let hundred_millis = Duration::from_millis(100);
        while idx_signal < 10 {
            let storage = daemon.lock().unwrap().storage.clone();
            let has_new_info = storage.lock().unwrap().new_info.load(Ordering::SeqCst);
            if has_new_info {
                break;
            }
            thread::sleep(hundred_millis);
            idx_signal += 1;
            if idx_signal == 10 {
                break;
            }
        }
        assert!(idx_signal == 10);
        teardown();
        daemon.lock().unwrap().stop();
        let _ = daemon_thread.join();
    }

    #[test]
    // Scenario
    // 1. A user authorizes another device to link
    // 2. The other device link to this user from a bridge
    // 3. Another device ask a user to link
    // 4. a user accepts.
    fn server_link_device_with_bridge() {
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
            device_author: Device {
                id: 1,
                name: String::new(),
                ring_id: String::from("Atlas_id1"),
                is_bridge: false
            },
            body: String::from("/register Atlas"),
            datatype: String::from("rori/command"),
            time: time::now(),
            metadatas: HashMap::new(),
        });
        assert!(server.anonymous_user.devices.len() == 2);
        assert!(server.registered_users.len() == 1);

        // Add a bridge with one user and ask linking
        server.handle_interaction(Interaction {
            device_author: Device {
                id: 2,
                name: String::new(),
                ring_id: String::from("Atlas_id2"),
                is_bridge: true
            },
            body: String::from("/bridgify"),
            datatype: String::from("rori/command"),
            time: time::now(),
            metadatas: HashMap::new(),
        });
        server.handle_interaction(Interaction {
            device_author: Device {
                id: 2,
                name: String::new(),
                ring_id: String::from("Atlas_id2"),
                is_bridge: true
            },
            body: String::from("/link Atlas"),
            datatype: String::from("rori/command"),
            time: time::now(),
            metadatas: HashMap::new(),
        });

        // Atlas_id authorizes Atlas_id2 to link
        server.handle_interaction(Interaction {
            device_author: Device {
                id: 1,
                name: String::new(),
                ring_id: String::from("Atlas_id1"),
                is_bridge: false
            },
            body: String::from("/link Atlas_id2"),
            datatype: String::from("rori/command"),
            time: time::now(),
            metadatas: HashMap::new(),
        });

        // The bridge should be anonymous + Atlas now!
        assert!(server.anonymous_user.devices.len() == 2);
        assert!(server.anonymous_user.devices.first().unwrap().ring_id == "Atlas_id3");
        assert!(server.anonymous_user.devices.last().unwrap().ring_id == "Atlas_id2");
        assert!(server.registered_users.first().unwrap().devices.len() == 2);

        teardown();
        daemon.lock().unwrap().stop();
        let _ = daemon_thread.join();
    }

     #[test]
    // Scenario
    // 1. A User revoke one device from a bridge
    fn server_remove_device_from_bridge() {
        let daemon = Arc::new(Mutex::new(Daemon::new()));
        let cloned_daemon = daemon.clone();
        let daemon_thread = thread::spawn(move|| {
            Daemon::run(cloned_daemon);
        });
        let mut atlas = User::new();
        atlas.name = String::from("Atlas");
        let mut atlas_device = Device::new(&2, &String::from("Atlas_id1"));
        atlas_device.name = String::from("Device");
        atlas.devices.push(atlas_device);
        let mut atlas_device = Device::new(&3, &String::from("Atlas_id2"));
        atlas_device.name = String::from("Device2");
        atlas.devices.push(atlas_device);
        let mut users = Vec::new();
        users.push(atlas);
        let mut server = setup(User::new(), users);
        server.add_new_anonymous_device(&String::from("Atlas_id2"));

        // Add a bridge with one user
        server.handle_interaction(Interaction {
            device_author: Device {
                id: 1,
                name: String::new(),
                ring_id: String::from("Atlas_id2"),
                is_bridge: true
            },
            body: String::from("/bridgify"),
            datatype: String::from("rori/command"),
            time: time::now(),
            metadatas: HashMap::new(),
        });

        let _ = Database::insert_new_device(&String::from("Atlas_id1"), &String::from("Atlas"), &String::from("Device"), false);
        let _ = Database::insert_new_device(&String::from("Atlas_id2"), &String::from("Atlas"), &String::from("Device2"), true);
        let _ = Database::update_sub_author(&3, &String::from("Atlas"));
        assert!(server.anonymous_user.devices.len() == 1);

        // Atlas_id2 do a /unregister (should succeed)

        let mut metadatas: HashMap<String, String> = HashMap::new();
        metadatas.insert(String::from("sa"), String::from("Atlas"));

        server.handle_interaction(Interaction {
            device_author: Device {
                id: 3,
                name: String::from("Device2"),
                ring_id: String::from("Atlas_id2"),
                is_bridge: false
            },
            body: String::from("/rm_device"),
            datatype: String::from("rori/command"),
            time: time::now(),
            metadatas: metadatas
        });

        assert!(server.anonymous_user.devices.len() == 1);
        assert!(server.anonymous_user.devices.first().unwrap().ring_id == "Atlas_id2");
        assert!(server.registered_users.len() == 1);
        assert!(server.registered_users.first().unwrap().devices.first().unwrap().ring_id == "Atlas_id1");

        teardown();
        daemon.lock().unwrap().stop();
        let _ = daemon_thread.join();
    }

     #[test]
    // Scenario
    // 1. A User unregister from a bridge
    fn server_unregister_from_bridge() {
        let daemon = Arc::new(Mutex::new(Daemon::new()));
        let cloned_daemon = daemon.clone();
        let daemon_thread = thread::spawn(move|| {
            Daemon::run(cloned_daemon);
        });
        let mut atlas = User::new();
        atlas.name = String::from("Atlas");
        let mut atlas_device = Device::new(&2, &String::from("Atlas_id1"));
        atlas_device.name = String::from("Device");
        atlas.devices.push(atlas_device);
        let mut atlas_device = Device::new(&3, &String::from("Atlas_id2"));
        atlas_device.name = String::from("Device2");
        atlas.devices.push(atlas_device);
        let mut users = Vec::new();
        users.push(atlas);
        let mut server = setup(User::new(), users);
        server.add_new_anonymous_device(&String::from("Atlas_id2"));

        // Add a bridge with one user
        server.handle_interaction(Interaction {
            device_author: Device {
                id: 1,
                name: String::new(),
                ring_id: String::from("Atlas_id2"),
                is_bridge: true
            },
            body: String::from("/bridgify"),
            datatype: String::from("rori/command"),
            time: time::now(),
            metadatas: HashMap::new(),
        });

        let _ = Database::insert_new_device(&String::from("Atlas_id1"), &String::from("Atlas"), &String::from("Device"), false);
        let _ = Database::insert_new_device(&String::from("Atlas_id2"), &String::from("Atlas"), &String::from("Device2"), true);
        let _ = Database::update_sub_author(&3, &String::from("Atlas"));
        assert!(server.anonymous_user.devices.len() == 1);

        // Atlas_id2 do a /unregister (should succeed)

        let mut metadatas: HashMap<String, String> = HashMap::new();
        metadatas.insert(String::from("sa"), String::from("Atlas"));

        server.handle_interaction(Interaction {
            device_author: Device {
                id: 3,
                name: String::from("Device2"),
                ring_id: String::from("Atlas_id2"),
                is_bridge: false
            },
            body: String::from("/unregister"),
            datatype: String::from("rori/command"),
            time: time::now(),
            metadatas: metadatas
        });

        assert!(server.anonymous_user.devices.len() == 2);
        assert!(server.anonymous_user.devices.first().unwrap().ring_id == "Atlas_id2");
        assert!(server.registered_users.len() == 0);

        teardown();
        daemon.lock().unwrap().stop();
        let _ = daemon_thread.join();
    }
}
