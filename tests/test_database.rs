extern crate core;
#[cfg(test)]
mod tests_database {
    use core::rori::database::Database;
    use core::rori::scheduler::ScheduledTask;
    use std::collections::HashMap;
    use std::fs;

    fn setup() {
        let _ = fs::remove_file("rori.db");
        Database::init_db(); // assert this function is correct.
    }

    fn teardown() {
        let _ = fs::remove_file("rori.db");
    }

    #[test]
    fn test_insert_new_device() {
        setup();
        // Insert new device
        let row = Database::insert_new_device(&String::from("GLaDOs"), &String::from("PBody"), &String::from("Atlas"), false);
        assert!(row.is_ok());
        // id should be unique
        let row = Database::insert_new_device(&String::from("GLaDOs"), &String::from("PBody"), &String::from("Atlas"), false);
        assert!(!row.is_ok());
        // Should be retrieven
        let devices = Database::get_devices();
        assert!(devices.len() == 1);
        let device = devices.first().unwrap();
        assert!(device.1 == "GLaDOs");
        assert!(device.2 == "PBody");
        assert!(device.3 == "Atlas");
        // Can handle more devices
        let row = Database::insert_new_device(&String::from("Tars"), &String::from(""), &String::from("Atlas"), false);
        assert!(row.is_ok());
        let devices = Database::get_devices();
        assert!(devices.len() == 2);
        teardown();
    }

    #[test]
    fn test_get_device() {
        setup();
        // Insert new devices
        let row = Database::insert_new_device(&String::from("GLaDOs"), &String::from("PBody"), &String::from("Atlas"), false);
        assert!(row.is_ok());
        let row = Database::insert_new_device(&String::from("Tars"), &String::from(""), &String::from("alexa"), false);
        assert!(row.is_ok());
        // Should be retrieven
        let device = Database::get_device(&String::from("GLaDOs"), &String::from("PBody"));
        assert!(device.1 == "GLaDOs");
        assert!(device.2 == "PBody");
        assert!(device.3 == "Atlas");
        // Can handle more devices
        let device = Database::get_device(&String::from("Tars"), &String::new());
        assert!(device.1 == "Tars");
        assert!(device.2 == "");
        assert!(device.3 == "alexa");

        // If device doesn't exist.
        let device = Database::get_device(&String::from("Eve"), &String::new());
        assert!(device.1 == "");
        assert!(device.2 == "");
        assert!(device.3 == "");
        teardown();
    }

    #[test]
    fn test_get_devices() {
        setup();
        // No devices, should be empty
        assert!(Database::get_devices().len() == 0);
        // Insert new devices
        let row = Database::insert_new_device(&String::from("GLaDOs"), &String::from("PBody"), &String::from("Atlas"), false);
        assert!(row.is_ok());
        let row = Database::insert_new_device(&String::from("Tars"), &String::from(""), &String::from("alexa"), false);
        assert!(row.is_ok());
        // Retrieve all devices and no more
        for (_, id, username, devicename, _) in Database::get_devices() {
            if id == "GLaDOs" {
                assert!(username == "PBody");
                assert!(devicename == "Atlas");
            } else if id == "Tars" {
                assert!(username == "");
                assert!(devicename == "alexa");
            } else {
                panic!("Unknown user");
            }
        }
        teardown();
    }

    #[test]
    fn test_rm_device() {
        setup();
        // Remove should do nothing if no row to remove
        let row = Database::insert_new_device(&String::from("Tars"), &String::from(""), &String::from("alexa"), false);
        assert!(row.is_ok());
        // No devices, should be empty
        assert!(Database::get_devices().len() == 1);
        let row = Database::remove_device(&2);
        assert!(row.is_ok());
        assert!(Database::get_devices().len() == 1);
        // Insert new device
        let row = Database::insert_new_device(&String::from("GLaDOs"), &String::from("PBody"), &String::from("Atlas"), false);
        assert!(row.is_ok());
        // Remove should succeed
        assert!(Database::get_devices().len() == 2);
        let row = Database::remove_device(&2);
        assert!(row.is_ok());
        // And get_device fails
        let device = Database::get_device(&String::from("GLaDOs"), &String::from("PBody"));
        assert!(device.1 == "");
        assert!(device.2 == "");
        assert!(device.3 == "");
        assert!(Database::get_devices().len() == 1);
        teardown();
    }

    #[test]
    fn test_search_device() {
        setup();
        // Search should return false if no device found
        let found = Database::search_devicename(&String::from("PBody"), &String::from("Atlas"));
        assert!(!found);
        // Insert new devices
        let row = Database::insert_new_device(&String::from("GLaDOs"), &String::from("PBody"), &String::from("Atlas"), false);
        assert!(row.is_ok());
        let row = Database::insert_new_device(&String::from("Tars"), &String::from(""), &String::from("alexa"), false);
        assert!(row.is_ok());
        // Search should succeed
        let found = Database::search_devicename(&String::from("PBody"), &String::from("Atlas"));
        assert!(found);
        teardown();
    }

    #[test]
    fn test_search_ring_id() {
        setup();
        // Search should return false if no device found
        let found = Database::search_hash(&String::from("GLaDOs"));
        assert!(!found);
        // Insert new devices
        let row = Database::insert_new_device(&String::from("GLaDOs"), &String::from("PBody"), &String::from("Atlas"), false);
        assert!(row.is_ok());
        let row = Database::insert_new_device(&String::from("Tars"), &String::from(""), &String::from("alexa"), false);
        assert!(row.is_ok());
        // Search should succeed
        let found = Database::search_hash(&String::from("GLaDOs"));
        assert!(found);
        teardown();
    }

    #[test]
    fn test_search_username() {
        setup();
        // Search should return false if no device found
        let found = Database::search_username(&String::from("PBody"));
        assert!(!found);
        // Insert new devices
        let row = Database::insert_new_device(&String::from("GLaDOs"), &String::from("PBody"), &String::from("Atlas"), false);
        assert!(row.is_ok());
        let row = Database::insert_new_device(&String::from("Tars"), &String::from(""), &String::from("alexa"), false);
        assert!(row.is_ok());
        // Search should succeed
        let found = Database::search_username(&String::from("PBody"));
        assert!(found);

        // RORI should be found
        let found = Database::search_username(&String::from("RORI"));
        assert!(found);
        let found = Database::search_username(&String::from("RoRI"));
        assert!(found);
        let found = Database::search_username(&String::from("Rori"));
        assert!(found);
        let found = Database::search_username(&String::from("rori"));
        assert!(found);
        teardown();
    }

    #[test]
    fn test_update_username() {
        setup();
        // Insert new devices
        let row = Database::insert_new_device(&String::from("GLaDOs"), &String::from("PBody"), &String::from("Atlas"), false);
        assert!(row.is_ok());
        let row = Database::insert_new_device(&String::from("Tars"), &String::from(""), &String::from("alexa"), false);
        assert!(row.is_ok());
        // Update should update
        let row = Database::update_username(&1, &String::from("BPody"));
        assert!(row.is_ok());
        // Should be retrieven
        let device = Database::get_device(&String::from("GLaDOs"), &String::from("BPody"));
        assert!(device.1 == "GLaDOs");
        assert!(device.2 == "BPody");
        assert!(device.3 == "Atlas");
        teardown();
    }

    #[test]
    fn test_update_devicename() {
        setup();
        // Insert new devices
        let row = Database::insert_new_device(&String::from("GLaDOs"), &String::from("PBody"), &String::from("Atlas"), false);
        assert!(row.is_ok());
        let row = Database::insert_new_device(&String::from("Tars"), &String::from(""), &String::from("alexa"), false);
        assert!(row.is_ok());
        // Update should update
        let row = Database::update_devicename(&1, &String::from("Jupiter"));
        assert!(row.is_ok());
        // Should be retrieven
        let device = Database::get_device(&String::from("GLaDOs"), &String::from("PBody"));
        assert!(device.1 == "GLaDOs");
        assert!(device.2 == "PBody");
        assert!(device.3 == "Jupiter");
        teardown();
    }

    #[test]
    fn test_get_set_datatypes() {
        setup();
        // Invalid set, should get nothing
        let mut dts = Vec::new();
        dts.push(String::from("dt1"));
        dts.push(String::from("dt2"));
        let _ = Database::set_datatypes(&1, dts.clone());
        let get = Database::get_datatypes(&1);
        assert!(get.len() == 0);
        // Insert new device
        let row = Database::insert_new_device(&String::from("GLaDOs"), &String::from("PBody"), &String::from("Atlas"), false);
        assert!(row.is_ok());
        let get = Database::get_datatypes(&1);
        assert!(get.len() == 0);
        // Valid set should get what we set
        let _ = Database::set_datatypes(&1, dts.clone());
        let get = Database::get_datatypes(&1);
        assert!(get == dts);
        // Should be rewritable
        let _ = Database::set_datatypes(&1, Vec::new());
        let get = Database::get_datatypes(&1);
        assert!(get.len() == 0);
        teardown();
    }

    #[test]
    fn test_is_bridge_with_username() {
        setup();
        // Insert new device
        let row = Database::insert_new_device(&String::from("GLaDOs"), &String::from("PBody"), &String::from("Atlas"), true);
        assert!(row.is_ok());
        assert!(Database::is_bridge_with_username(&String::from("GLaDOs"), &String::from("PBody")));
        assert!(!Database::is_bridge_with_username(&String::from("GLaDOs"), &String::from("NotPBody")));
        teardown();
    }

    #[test]
    fn test_sub_author() {
        setup();
        // Insert new device
        let row = Database::insert_new_device(&String::from("GLaDOs"), &String::from("PBody"), &String::from("Atlas"), true);
        assert!(row.is_ok());
        // Update sub author
        assert!(Database::sub_author(&String::from("GLaDOs"), &String::from("Asimo")) == String::new());
        let row = Database::update_sub_author(&1, &String::from("Asimo"));
        assert!(row.is_ok());
        assert!(Database::sub_author(&String::from("GLaDOs"), &String::from("Asimo")) == String::from("PBody"));
        teardown();
    }

    #[test]
    fn test_get_devices_for_username() {
        setup();
        // Insert new devices
        let row = Database::insert_new_device(&String::from("GLaDOs"), &String::from("PBody"), &String::from("Atlas"), false);
        assert!(row.is_ok());
        let row = Database::update_sub_author(&1, &String::from("Asimo"));
        assert!(row.is_ok());
        let row = Database::insert_new_device(&String::from("Weasley"), &String::from("PBody"), &String::from("Asimo"), false);
        assert!(row.is_ok());
        let row = Database::insert_new_device(&String::from("Eve"), &String::from("Wall-E"), &String::from(""), false);
        assert!(row.is_ok());
        // Shoud get two devices
        let user_devices = Database::get_devices_for_username(&String::from("PBody"));
        assert!(user_devices.len() == 2);
        assert!(user_devices[0].0 == 1);
        assert!(user_devices[1].0 == 2);
        teardown();
    }

    #[test]
    fn test_insert_bridge_device_twice() {
        setup();
        // Insert new devices
        let row = Database::insert_new_device(&String::from("GLaDOs"), &String::from("PBody"), &String::from("Atlas"), true);
        assert!(row.is_ok());
        let row = Database::insert_new_device(&String::from("GLaDOs"), &String::from("PBody"), &String::from("Atlas"), true);
        assert!(!row.is_ok());
        teardown();
    }

    #[test]
    fn test_get_modules_datatypes() {
        setup();
        // Insert modules datatypes
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let row = conn.execute("INSERT INTO modules (name, priority, enabled, type, condition, path)
                                VALUES (\"foo\", 1, 1, \"foo\", \"foo\", \"foo\")", rusqlite::NO_PARAMS);
        assert!(row.is_ok());
        // Get foo in datatypes
        let datatypes = Database::get_modules_datatypes();
        assert!(datatypes.len() == 3);
        assert!(datatypes[0] == String::from("text/plain"));
        assert!(datatypes[1] == String::from("rori/command"));
        assert!(datatypes[2] == String::from("foo"));
        teardown();
    }

    #[test]
    fn test_get_module() {
        setup();
        // Insert module foo
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let row = conn.execute("INSERT INTO modules (name, priority, enabled, type, condition, path)
                                VALUES (\"foo\", 1, 1, \"foo\", \"foo\", \"foo\")", rusqlite::NO_PARAMS);
        assert!(row.is_ok());
        // Get foo in modules
        let module = Database::get_module(&(row.unwrap() as i32));
        assert!(!module.is_none());
        assert!(module.unwrap().name == String::from("foo"));
        teardown();
    }

    #[test]
    fn test_get_module_fail() {
        setup();
        // Get foo in modules
        let module = Database::get_module(&1);
        assert!(module.is_none());
        teardown();
    }

    #[test]
    fn test_get_module_id_by_name() {
        setup();
        // Insert module foo
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let row = conn.execute("INSERT INTO modules (name, priority, enabled, type, condition, path)
                                VALUES (\"foo\", 1, 1, \"foo\", \"foo\", \"foo\")", rusqlite::NO_PARAMS);
        assert!(row.is_ok());
        // Get foo in modules
        let module = Database::get_module_id_by_name(&String::from("foo"));
        assert!(module != 0);
        assert!(module == row.unwrap() as i32);
        teardown();
    }

    #[test]
    fn test_get_module_id_by_name_fail() {
        setup();
        // Get foo in modules
        let module = Database::get_module_id_by_name(&String::from("foo"));
        assert!(module == 0);
        teardown();
    }

    #[test]
    fn test_get_tasks_no_tasks() {
        setup();
        // test get tasks no task
        let tasks = Database::get_tasks();
        assert!(tasks.len() == 0);
        teardown();
    }

    #[test]
    fn test_add_get_tasks() {
        setup();
        // Insert module foo
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let row = conn.execute("INSERT INTO modules (name, priority, enabled, type, condition, path)
                                VALUES (\"foo\", 1, 1, \"foo\", \"foo\", \"foo\")", rusqlite::NO_PARAMS);
        let module_id = row.unwrap() as i32;
        let task1 = ScheduledTask {
            id : 0,
            module : module_id,
            parameter : String::new(),
            at : String::new(),
            seconds : 0,
            minutes : 0,
            hours : 0,
            days : String::new(),
            repeat : false
        };
        let task2 = ScheduledTask {
            id : 0,
            module : module_id,
            parameter : String::new(),
            at : String::new(),
            seconds : 1,
            minutes : 2,
            hours : 0,
            days : String::new(),
            repeat : false
        };
        Database::add_task(&task1);
        Database::add_task(&task2);
        // test get tasks
        let tasks = Database::get_tasks();
        assert!(tasks.len() == 2);
        assert!(tasks[0].seconds == task1.seconds);
        assert!(tasks[1].seconds == task2.seconds);
        teardown();
    }

    #[test]
    fn test_get_task() {
        setup();
        // Insert module foo
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let row = conn.execute("INSERT INTO modules (name, priority, enabled, type, condition, path)
                                VALUES (\"foo\", 1, 1, \"foo\", \"foo\", \"foo\")", rusqlite::NO_PARAMS);
        let module_id = row.unwrap() as i32;
        let task1 = ScheduledTask {
            id : 0,
            module : module_id,
            parameter : String::new(),
            at : String::new(),
            seconds : 0,
            minutes : 0,
            hours : 0,
            days : String::new(),
            repeat : false
        };
        let task2 = ScheduledTask {
            id : 0,
            module : module_id,
            parameter : String::new(),
            at : String::new(),
            seconds : 1,
            minutes : 2,
            hours : 0,
            days : String::new(),
            repeat : false
        };
        Database::add_task(&task1);
        Database::add_task(&task2);
        // test get task
        let task = Database::get_task(&1);
        assert!(!task.is_none());
        assert!(task.as_ref().unwrap().id == 1);
        assert!(task.as_ref().unwrap().seconds == task1.seconds);
        teardown();
    }

    #[test]
    fn test_get_task_failed() {
        setup();
        // test get task fails
        let task = Database::get_task(&1);
        assert!(task.is_none());
        teardown();
    }

    #[test]
    fn test_search_no_parameter() {
        setup();
        // Insert module foo
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let row = conn.execute("INSERT INTO modules (name, priority, enabled, type, condition, path)
                                VALUES (\"foo\", 1, 1, \"foo\", \"foo\", \"foo\")", rusqlite::NO_PARAMS);
        let module_id = row.unwrap() as i32;
        let task1 = ScheduledTask {
            id : 0,
            module : module_id,
            parameter : String::from("{\"author\":\"foo\",\"param\":\"bar\"}"),
            at : String::new(),
            seconds : 56,
            minutes : 0,
            hours : 0,
            days : String::new(),
            repeat : false
        };
        Database::add_task(&task1);
        // test get search task no parameter
        let task = Database::search_task(&String::from("1"), HashMap::new());
        assert!(!task.is_none());
        assert!(task.as_ref().unwrap().id == 1);
        assert!(task.as_ref().unwrap().seconds == task1.seconds);
        teardown();
    }

    #[test]
    fn test_search_sub_parameter() {
        setup();
        // Insert module foo
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let row = conn.execute("INSERT INTO modules (name, priority, enabled, type, condition, path)
                                VALUES (\"foo\", 1, 1, \"foo\", \"foo\", \"foo\")", rusqlite::NO_PARAMS);
        let module_id = row.unwrap() as i32;
        let task1 = ScheduledTask {
            id : 0,
            module : module_id,
            parameter : String::from("{\"author\":\"foo\",\"param\":\"bar\"}"),
            at : String::new(),
            seconds : 56,
            minutes : 0,
            hours : 0,
            days : String::new(),
            repeat : false
        };
        Database::add_task(&task1);
        // test get search task sub parameter
        let mut parameter = HashMap::new();
        parameter.insert(String::from("author"), String::from("foo"));
        let task = Database::search_task(&String::from("1"), parameter);
        assert!(!task.is_none());
        assert!(task.as_ref().unwrap().id == 1);
        assert!(task.as_ref().unwrap().seconds == task1.seconds);
        teardown();
    }

    #[test]
    fn test_search_exact_parameter() {
        setup();
        // Insert module foo
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let row = conn.execute("INSERT INTO modules (name, priority, enabled, type, condition, path)
                                VALUES (\"foo\", 1, 1, \"foo\", \"foo\", \"foo\")", rusqlite::NO_PARAMS);
        let module_id = row.unwrap() as i32;
        let task1 = ScheduledTask {
            id : 0,
            module : module_id,
            parameter : String::from("{\"author\":\"foo\",\"param\":\"bar\"}"),
            at : String::new(),
            seconds : 56,
            minutes : 0,
            hours : 0,
            days : String::new(),
            repeat : false
        };
        Database::add_task(&task1);
        // test get search task exact parameter
        let mut parameter = HashMap::new();
        parameter.insert(String::from("author"), String::from("foo"));
        parameter.insert(String::from("param"), String::from("bar"));
        let task = Database::search_task(&String::from("1"), parameter);
        assert!(!task.is_none());
        assert!(task.as_ref().unwrap().id == 1);
        assert!(task.as_ref().unwrap().seconds == task1.seconds);
        teardown();
    }

    #[test]
    fn test_search_too_many_parameter() {
        setup();
        // Insert module foo
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let row = conn.execute("INSERT INTO modules (name, priority, enabled, type, condition, path)
                                VALUES (\"foo\", 1, 1, \"foo\", \"foo\", \"foo\")", rusqlite::NO_PARAMS);
        let module_id = row.unwrap() as i32;
        let task1 = ScheduledTask {
            id : 0,
            module : module_id,
            parameter : String::from("{\"author\":\"foo\",\"param\":\"bar\"}"),
            at : String::new(),
            seconds : 56,
            minutes : 0,
            hours : 0,
            days : String::new(),
            repeat : false
        };
        Database::add_task(&task1);
        // test get search task too maty parameter
        let mut parameter = HashMap::new();
        parameter.insert(String::from("author"), String::from("foo"));
        parameter.insert(String::from("param"), String::from("bar"));
        parameter.insert(String::from("param2"), String::from("bar2"));
        let task = Database::search_task(&String::from("1"), parameter);
        assert!(task.is_none());
        teardown();
    }

    #[test]
    fn test_search_not_found() {
        setup();
        // Insert module foo
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let row = conn.execute("INSERT INTO modules (name, priority, enabled, type, condition, path)
                                VALUES (\"foo\", 1, 1, \"foo\", \"foo\", \"foo\")", rusqlite::NO_PARAMS);
        let module_id = row.unwrap() as i32;
        let task1 = ScheduledTask {
            id : 0,
            module : module_id,
            parameter : String::from("{\"author\":\"foo\",\"param\":\"bar\"}"),
            at : String::new(),
            seconds : 56,
            minutes : 0,
            hours : 0,
            days : String::new(),
            repeat : false
        };
        Database::add_task(&task1);
        // test get search task not found
        let mut parameter = HashMap::new();
        parameter.insert(String::from("author"), String::from("foo"));
        parameter.insert(String::from("param"), String::from("bar"));
        let task = Database::search_task(&String::from("2"), parameter);
        assert!(task.is_none());
        teardown();
    }

    #[test]
    fn test_search_not_found_param() {
        setup();
        // Insert module foo
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let row = conn.execute("INSERT INTO modules (name, priority, enabled, type, condition, path)
                                VALUES (\"foo\", 1, 1, \"foo\", \"foo\", \"foo\")", rusqlite::NO_PARAMS);
        let module_id = row.unwrap() as i32;
        let task1 = ScheduledTask {
            id : 0,
            module : module_id,
            parameter : String::from("{\"author\":\"foo\",\"param\":\"bar\"}"),
            at : String::new(),
            seconds : 56,
            minutes : 0,
            hours : 0,
            days : String::new(),
            repeat : false
        };
        Database::add_task(&task1);
        // test get search task not found parameter
        let mut parameter = HashMap::new();
        parameter.insert(String::from("author"), String::from("bar"));
        parameter.insert(String::from("param"), String::from("foo"));
        let task = Database::search_task(&String::from("1"), parameter);
        assert!(task.is_none());
        teardown();
    }

    #[test]
    fn test_update_task() {
        setup();
        // Insert module foo
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let row = conn.execute("INSERT INTO modules (name, priority, enabled, type, condition, path)
                                VALUES (\"foo\", 1, 1, \"foo\", \"foo\", \"foo\")", rusqlite::NO_PARAMS);
        let module_id = row.unwrap() as i32;
        let task1 = ScheduledTask {
            id : 0,
            module : module_id,
            parameter : String::new(),
            at : String::new(),
            seconds : 0,
            minutes : 0,
            hours : 0,
            days : String::new(),
            repeat : false
        };
        let task2 = ScheduledTask {
            id : 0,
            module : module_id,
            parameter : String::new(),
            at : String::new(),
            seconds : 1,
            minutes : 2,
            hours : 0,
            days : String::new(),
            repeat : false
        };
        Database::add_task(&task1);
        Database::add_task(&task2);
        let task3 = ScheduledTask {
            id : 1,
            module : module_id,
            parameter : String::new(),
            at : String::new(),
            seconds : 1,
            minutes : 3,
            hours : 4,
            days : String::new(),
            repeat : false
        };
        // test update task
        let _ = Database::update_task(&task3);
        let task_retrieven = Database::get_task(&1);
        assert!(!task_retrieven.is_none());
        assert!(task_retrieven.as_ref().unwrap().id == 1);
        assert!(task_retrieven.unwrap().seconds == task3.seconds);
        let tasks = Database::get_tasks();
        assert!(tasks.len() == 2);
        teardown();
    }


    #[test]
    fn test_update_task_failed() {
        setup();
        // Insert module foo
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let row = conn.execute("INSERT INTO modules (name, priority, enabled, type, condition, path)
                                VALUES (\"foo\", 1, 1, \"foo\", \"foo\", \"foo\")", rusqlite::NO_PARAMS);
        let module_id = row.unwrap() as i32;
        let task1 = ScheduledTask {
            id : 0,
            module : module_id,
            parameter : String::new(),
            at : String::new(),
            seconds : 0,
            minutes : 0,
            hours : 0,
            days : String::new(),
            repeat : false
        };
        let task2 = ScheduledTask {
            id : 0,
            module : module_id,
            parameter : String::new(),
            at : String::new(),
            seconds : 1,
            minutes : 2,
            hours : 0,
            days : String::new(),
            repeat : false
        };
        Database::add_task(&task1);
        Database::add_task(&task2);
        let task3 = ScheduledTask {
            id : 196,
            module : module_id,
            parameter : String::new(),
            at : String::new(),
            seconds : 1,
            minutes : 3,
            hours : 4,
            days : String::new(),
            repeat : false
        };
        // test update task failed
        let _ = Database::update_task(&task3);
        let task_retrieven = Database::get_task(&1);
        assert!(task_retrieven.as_ref().unwrap().id == 1);
        assert!(task_retrieven.unwrap().seconds == 0);
        let tasks = Database::get_tasks();
        assert!(tasks.len() == 2);
        teardown();
    }

    #[test]
    fn test_rm_task() {
        setup();
        // Insert module foo
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let row = conn.execute("INSERT INTO modules (name, priority, enabled, type, condition, path)
                                VALUES (\"foo\", 1, 1, \"foo\", \"foo\", \"foo\")", rusqlite::NO_PARAMS);
        let module_id = row.unwrap() as i32;
        let task1 = ScheduledTask {
            id : 0,
            module : module_id,
            parameter : String::new(),
            at : String::new(),
            seconds : 0,
            minutes : 0,
            hours : 0,
            days : String::new(),
            repeat : false
        };
        let task2 = ScheduledTask {
            id : 0,
            module : module_id,
            parameter : String::new(),
            at : String::new(),
            seconds : 1,
            minutes : 2,
            hours : 0,
            days : String::new(),
            repeat : false
        };
        let id1 = Database::add_task(&task1);
        Database::add_task(&task2);
        // test rm task
        let task_rm = Database::rm_task(&id1.unwrap());
        assert!(task_rm.is_ok());
        assert!(task_rm.unwrap() as i32 == id1.unwrap());
        let tasks = Database::get_tasks();
        assert!(tasks.len() == 1);
        assert!(tasks[0].seconds == task2.seconds);
        teardown();
    }

    #[test]
    fn test_rm_task_failed() {
        setup();
        // test rm task fails
        let task = Database::rm_task(&1);
        assert!(task.unwrap() == 0);
        teardown();
    }

}