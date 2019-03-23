extern crate core;
#[cfg(test)]
mod tests_database {
    use core::rori::database::Database;
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
}