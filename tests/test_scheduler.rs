extern crate core;
#[cfg(test)]
mod tests_scheduler {
    use core::rori::database::Database;
    use core::rori::scheduler::{Scheduler, ScheduledTask};
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
    fn test_init_scheduler_ok() {
        setup();
        // Insert module foo
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let _ = conn.execute("INSERT INTO modules (name, priority, enabled, type, condition, path)
                                VALUES (\"foo\", 1, 1, \"foo\", \"foo\", \"foo\")", rusqlite::NO_PARAMS);
        let _ = Database::insert_new_device(&String::from("foo"), &String::from("bar"), &String::from("bar"), false);
        let task1 = ScheduledTask {
            id : 0,
            module : 1,
            parameter : String::from("{\"ring_id\":\"foo\",\"username\":\"bar\"}"),
            at : String::new(),
            seconds : 0,
            minutes : 0,
            hours : 0,
            days : String::new(),
            repeat : false
        };
        let task2 = ScheduledTask {
            id : 0,
            module : 1,
            parameter : String::from("{\"ring_id\":\"foo\",\"username\":\"bar\"}"),
            at : String::new(),
            seconds : 1,
            minutes : 2,
            hours : 0,
            days : String::new(),
            repeat : false
        };
        Database::add_task(&task1);
        Database::add_task(&task2);
        Scheduler::no_thread();
        let tasks = Database::get_tasks();
        assert!(tasks.len() == 2);
        teardown();
    }

    #[test]
    fn test_init_scheduler_ok_invalid_module() {
        setup();
        // Insert module foo
        let _ = Database::insert_new_device(&String::from("foo"), &String::from("bar"), &String::from("bar"), false);
        let task1 = ScheduledTask {
            id : 0,
            module : 1,
            parameter : String::from("{\"ring_id\":\"foo\",\"username\":\"bar\"}"),
            at : String::new(),
            seconds : 0,
            minutes : 0,
            hours : 0,
            days : String::new(),
            repeat : false
        };
        let task2 = ScheduledTask {
            id : 0,
            module : 1,
            parameter : String::from("{\"ring_id\":\"foo\",\"username\":\"bar\"}"),
            at : String::new(),
            seconds : 1,
            minutes : 2,
            hours : 0,
            days : String::new(),
            repeat : false
        };
        Database::add_task(&task1);
        Database::add_task(&task2);
        Scheduler::no_thread();
        let tasks = Database::get_tasks();
        assert!(tasks.len() == 0);
        teardown();
    }

    #[test]
    fn test_init_scheduler_ok_invalid_parameter() {
        setup();
        // Insert module foo
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let _ = conn.execute("INSERT INTO modules (name, priority, enabled, type, condition, path)
                                VALUES (\"foo\", 1, 1, \"foo\", \"foo\", \"foo\")", rusqlite::NO_PARAMS);
        let _ = Database::insert_new_device(&String::from("foo"), &String::from("bar"), &String::from("bar"), false);
        let task1 = ScheduledTask {
            id : 0,
            module : 1,
            parameter : String::from("{\"username\":\"bar\"}"),
            at : String::new(),
            seconds : 0,
            minutes : 0,
            hours : 0,
            days : String::new(),
            repeat : false
        };
        let task2 = ScheduledTask {
            id : 0,
            module : 1,
            parameter : String::from("{\"ring_id\":\"foo\"}"),
            at : String::new(),
            seconds : 1,
            minutes : 2,
            hours : 0,
            days : String::new(),
            repeat : false
        };
        Database::add_task(&task1);
        Database::add_task(&task2);
        Scheduler::no_thread();
        let tasks = Database::get_tasks();
        assert!(tasks.len() == 0);
        teardown();
    }

    #[test]
    fn test_init_scheduler_ok_invalid_device() {
        setup();
        // Insert module foo
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let _ = conn.execute("INSERT INTO modules (name, priority, enabled, type, condition, path)
                                VALUES (\"foo\", 1, 1, \"foo\", \"foo\", \"foo\")", rusqlite::NO_PARAMS);
        let task1 = ScheduledTask {
            id : 0,
            module : 1,
            parameter : String::from("{\"ring_id\":\"foo\",\"username\":\"bar\"}"),
            at : String::new(),
            seconds : 0,
            minutes : 0,
            hours : 0,
            days : String::new(),
            repeat : false
        };
        let task2 = ScheduledTask {
            id : 0,
            module : 1,
            parameter : String::from("{\"ring_id\":\"foo\",\"username\":\"bar\"}"),
            at : String::new(),
            seconds : 1,
            minutes : 2,
            hours : 0,
            days : String::new(),
            repeat : false
        };
        Database::add_task(&task1);
        Database::add_task(&task2);
        Scheduler::no_thread();
        let tasks = Database::get_tasks();
        assert!(tasks.len() == 0);
        teardown();
    }

    #[test]
    fn test_add_task() {
        setup();
        // Insert module foo
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let _ = conn.execute("INSERT INTO modules (name, priority, enabled, type, condition, path)
                                VALUES (\"foo\", 1, 1, \"foo\", \"foo\", \"foo\")", rusqlite::NO_PARAMS);
        let _ = Database::insert_new_device(&String::from("foo"), &String::from("bar"), &String::from("bar"), false);
        let mut task1 = HashMap::new();
        task1.insert(String::from("id"), String::from("0"));
        task1.insert(String::from("module"), String::from("1"));
        task1.insert(String::from("parameter"), String::from("{\"ring_id\":\"foo\",\"username\":\"bar\"}"));
        task1.insert(String::from("at"), String::new());
        task1.insert(String::from("seconds"), String::from("0"));
        task1.insert(String::from("minutes"), String::from("0"));
        task1.insert(String::from("hours"), String::from("0"));
        task1.insert(String::from("days"), String::new());
        task1.insert(String::from("repeat"), String::from("True"));
        let mut scheduler = Scheduler::no_thread();
        let task1 = serde_json::to_string(&task1).unwrap_or(String::new());
        scheduler.add_task(&task1);
        let tasks = Database::get_tasks();
        assert!(tasks.len() == 1);
        teardown();
    }

    #[test]
    fn test_add_invalid_task() {
        setup();
        // Insert module foo
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let _ = conn.execute("INSERT INTO modules (name, priority, enabled, type, condition, path)
                                VALUES (\"foo\", 1, 1, \"foo\", \"foo\", \"foo\")", rusqlite::NO_PARAMS);
        let _ = Database::insert_new_device(&String::from("foo"), &String::from("bar"), &String::from("bar"), false);
        let mut task1 = HashMap::new();
        task1.insert(String::from("id"), String::from("0"));
        task1.insert(String::from("module"), String::from("1"));
        task1.insert(String::from("at"), String::new());
        task1.insert(String::from("seconds"), String::from("0"));
        task1.insert(String::from("minutes"), String::from("0"));
        task1.insert(String::from("hours"), String::from("0"));
        task1.insert(String::from("days"), String::new());
        task1.insert(String::from("repeat"), String::from("True"));
        let mut scheduler = Scheduler::no_thread();
        let task1 = serde_json::to_string(&task1).unwrap_or(String::new());
        scheduler.add_task(&task1);
        let tasks = Database::get_tasks();
        assert!(tasks.len() == 0);
        teardown();
    }

    #[test]
    fn test_update_task() {
        setup();
        // Insert module foo
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let _ = conn.execute("INSERT INTO modules (name, priority, enabled, type, condition, path)
                                VALUES (\"foo\", 1, 1, \"foo\", \"foo\", \"foo\")", rusqlite::NO_PARAMS);
        let _ = Database::insert_new_device(&String::from("foo"), &String::from("bar"), &String::from("bar"), false);
        let task1 = ScheduledTask {
            id : 0,
            module : 1,
            parameter : String::from("{\"ring_id\":\"foo\",\"username\":\"bar\"}"),
            at : String::new(),
            seconds : 0,
            minutes : 0,
            hours : 0,
            days : String::new(),
            repeat : false
        };
        Database::add_task(&task1);
        let mut scheduler = Scheduler::no_thread();
        let pre_update_task = Database::get_tasks()[0].clone();

        let mut task1 = HashMap::new();
        task1.insert(String::from("id"), String::from("1"));
        task1.insert(String::from("module"), String::from("1"));
        task1.insert(String::from("parameter"), String::from("{\"ring_id\":\"foo\",\"username\":\"bar\"}"));
        task1.insert(String::from("at"), String::new());
        task1.insert(String::from("seconds"), String::from("0"));
        task1.insert(String::from("minutes"), String::from("54"));
        task1.insert(String::from("hours"), String::from("0"));
        task1.insert(String::from("days"), String::new());
        task1.insert(String::from("repeat"), String::from("True"));
        let task1 = serde_json::to_string(&task1).unwrap_or(String::new());
        scheduler.update_task(&task1);

        let new_tasks = Database::get_tasks();
        assert!(new_tasks.len() == 1);
        assert!(new_tasks[0] != pre_update_task);

        teardown();
    }

    #[test]
    fn test_update_task_invalid() {
        setup();
        // Insert module foo
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let _ = conn.execute("INSERT INTO modules (name, priority, enabled, type, condition, path)
                                VALUES (\"foo\", 1, 1, \"foo\", \"foo\", \"foo\")", rusqlite::NO_PARAMS);
        let _ = Database::insert_new_device(&String::from("foo"), &String::from("bar"), &String::from("bar"), false);
        let task1 = ScheduledTask {
            id : 0,
            module : 1,
            parameter : String::from("{\"ring_id\":\"foo\",\"username\":\"bar\"}"),
            at : String::new(),
            seconds : 0,
            minutes : 0,
            hours : 0,
            days : String::new(),
            repeat : false
        };
        Database::add_task(&task1);
        let mut scheduler = Scheduler::no_thread();
        let pre_update_task = Database::get_tasks()[0].clone();

        let mut task1 = HashMap::new();
        task1.insert(String::from("module"), String::from("1"));
        task1.insert(String::from("parameter"), String::from("{\"ring_id\":\"foo\",\"username\":\"bar\"}"));
        task1.insert(String::from("at"), String::new());
        task1.insert(String::from("seconds"), String::from("0"));
        task1.insert(String::from("minutes"), String::from("54"));
        task1.insert(String::from("hours"), String::from("0"));
        task1.insert(String::from("days"), String::new());
        task1.insert(String::from("repeat"), String::from("True"));
        let task1 = serde_json::to_string(&task1).unwrap_or(String::new());
        scheduler.update_task(&task1);

        let new_tasks = Database::get_tasks();
        assert!(new_tasks.len() == 1);
        assert!(new_tasks[0] == pre_update_task);

        teardown();
    }

    #[test]
    fn test_update_task_non_existing() {
        setup();
        // Insert module foo
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let _ = conn.execute("INSERT INTO modules (name, priority, enabled, type, condition, path)
                                VALUES (\"foo\", 1, 1, \"foo\", \"foo\", \"foo\")", rusqlite::NO_PARAMS);
        let _ = Database::insert_new_device(&String::from("foo"), &String::from("bar"), &String::from("bar"), false);
        let task1 = ScheduledTask {
            id : 0,
            module : 1,
            parameter : String::from("{\"ring_id\":\"foo\",\"username\":\"bar\"}"),
            at : String::new(),
            seconds : 0,
            minutes : 0,
            hours : 0,
            days : String::new(),
            repeat : false
        };
        Database::add_task(&task1);
        let mut scheduler = Scheduler::no_thread();
        let pre_update_task = Database::get_tasks()[0].clone();

        let mut task1 = HashMap::new();
        task1.insert(String::from("id"), String::from("1412"));
        task1.insert(String::from("module"), String::from("1"));
        task1.insert(String::from("parameter"), String::from("{\"ring_id\":\"foo\",\"username\":\"bar\"}"));
        task1.insert(String::from("at"), String::new());
        task1.insert(String::from("seconds"), String::from("0"));
        task1.insert(String::from("minutes"), String::from("54"));
        task1.insert(String::from("hours"), String::from("0"));
        task1.insert(String::from("days"), String::new());
        task1.insert(String::from("repeat"), String::from("True"));
        let task1 = serde_json::to_string(&task1).unwrap_or(String::new());
        scheduler.update_task(&task1);

        let new_tasks = Database::get_tasks();
        assert!(new_tasks.len() == 1);
        assert!(new_tasks[0] == pre_update_task);

        teardown();
    }

    #[test]
    fn test_rm_task() {
        setup();
        // Insert module foo
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let _ = conn.execute("INSERT INTO modules (name, priority, enabled, type, condition, path)
                                VALUES (\"foo\", 1, 1, \"foo\", \"foo\", \"foo\")", rusqlite::NO_PARAMS);
        let _ = Database::insert_new_device(&String::from("foo"), &String::from("bar"), &String::from("bar"), false);
        let task1 = ScheduledTask {
            id : 0,
            module : 1,
            parameter : String::from("{\"ring_id\":\"foo\",\"username\":\"bar\"}"),
            at : String::new(),
            seconds : 0,
            minutes : 0,
            hours : 0,
            days : String::new(),
            repeat : false
        };
        let task2 = ScheduledTask {
            id : 0,
            module : 1,
            parameter : String::from("{\"ring_id\":\"foo\",\"username\":\"bar\"}"),
            at : String::new(),
            seconds : 1,
            minutes : 2,
            hours : 0,
            days : String::new(),
            repeat : false
        };
        Database::add_task(&task1);
        Database::add_task(&task2);
        let mut scheduler = Scheduler::no_thread();
        let result = scheduler.rm_task(&1);
        assert!(!result.is_none());
        let tasks = Database::get_tasks();
        assert!(tasks.len() == 1);
        assert!(tasks[0].id == 2);
        teardown();
    }

    #[test]
    fn test_rm_task_non_existing() {
        setup();
        // Insert module foo
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let _ = conn.execute("INSERT INTO modules (name, priority, enabled, type, condition, path)
                                VALUES (\"foo\", 1, 1, \"foo\", \"foo\", \"foo\")", rusqlite::NO_PARAMS);
        let _ = Database::insert_new_device(&String::from("foo"), &String::from("bar"), &String::from("bar"), false);
        let task1 = ScheduledTask {
            id : 0,
            module : 1,
            parameter : String::from("{\"ring_id\":\"foo\",\"username\":\"bar\"}"),
            at : String::new(),
            seconds : 0,
            minutes : 0,
            hours : 0,
            days : String::new(),
            repeat : false
        };
        let task2 = ScheduledTask {
            id : 0,
            module : 1,
            parameter : String::from("{\"ring_id\":\"foo\",\"username\":\"bar\"}"),
            at : String::new(),
            seconds : 1,
            minutes : 2,
            hours : 0,
            days : String::new(),
            repeat : false
        };
        Database::add_task(&task1);
        Database::add_task(&task2);
        let mut scheduler = Scheduler::no_thread();
        scheduler.rm_task(&1412);
        let tasks = Database::get_tasks();
        assert!(tasks.len() == 2);
        teardown();
    }

}