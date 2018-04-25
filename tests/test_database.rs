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
        let row = Database::insert_new_device(&String::from("GLaDOs"), &String::from("PBody"), &String::from("Atlas"));
        assert!(row.is_ok());
        // id should be unique
        let row = Database::insert_new_device(&String::from("GLaDOs"), &String::from("PBody"), &String::from("Atlas"));
        assert!(!row.is_ok());
        // Should be retrieven
        let devices = Database::get_devices();
        assert!(devices.len() == 1);
        let device = devices.first().unwrap();
        assert!(device.0 == "GLaDOs");
        assert!(device.1 == "PBody");
        assert!(device.2 == "Atlas");
        // Can handle more devices
        let row = Database::insert_new_device(&String::from("Tars"), &String::from(""), &String::from("Atlas"));
        assert!(row.is_ok());
        let devices = Database::get_devices();
        assert!(devices.len() == 2);
        teardown();
    }

    #[test]
    fn test_get_device() {
        setup();
        // Insert new devices
        let row = Database::insert_new_device(&String::from("GLaDOs"), &String::from("PBody"), &String::from("Atlas"));
        assert!(row.is_ok());
        let row = Database::insert_new_device(&String::from("Tars"), &String::from(""), &String::from("alexa"));
        assert!(row.is_ok());
        // Should be retrieven
        let device = Database::get_device(&String::from("GLaDOs"));
        assert!(device.0 == "GLaDOs");
        assert!(device.1 == "PBody");
        assert!(device.2 == "Atlas");
        // Can handle more devices
        let device = Database::get_device(&String::from("Tars"));
        assert!(device.0 == "Tars");
        assert!(device.1 == "");
        assert!(device.2 == "alexa");

        // If device doesn't exist.
        let device = Database::get_device(&String::from("Eve"));
        assert!(device.0 == "");
        assert!(device.1 == "");
        assert!(device.2 == "");
        teardown();
    }

    #[test]
    fn test_get_devices() {
        setup();
        // No devices, should be empty
        assert!(Database::get_devices().len() == 0);
        // Insert new devices
        let row = Database::insert_new_device(&String::from("GLaDOs"), &String::from("PBody"), &String::from("Atlas"));
        assert!(row.is_ok());
        let row = Database::insert_new_device(&String::from("Tars"), &String::from(""), &String::from("alexa"));
        assert!(row.is_ok());
        // Retrieve all devices and no more
        for (id, username, devicename) in Database::get_devices() {
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
        let row = Database::insert_new_device(&String::from("Tars"), &String::from(""), &String::from("alexa"));
        assert!(row.is_ok());
        // No devices, should be empty
        assert!(Database::get_devices().len() == 1);
        let row = Database::remove_device(&String::from("GLaDOs"));
        assert!(row.is_ok());
        assert!(Database::get_devices().len() == 1);
        // Insert new device
        let row = Database::insert_new_device(&String::from("GLaDOs"), &String::from("PBody"), &String::from("Atlas"));
        assert!(row.is_ok());
        // Remove should succeed
        assert!(Database::get_devices().len() == 2);
        let row = Database::remove_device(&String::from("GLaDOs"));
        assert!(row.is_ok());
        // And get_device fails
        let device = Database::get_device(&String::from("GLaDOs"));
        assert!(device.0 == "");
        assert!(device.1 == "");
        assert!(device.2 == "");
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
        let row = Database::insert_new_device(&String::from("GLaDOs"), &String::from("PBody"), &String::from("Atlas"));
        assert!(row.is_ok());
        let row = Database::insert_new_device(&String::from("Tars"), &String::from(""), &String::from("alexa"));
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
        let found = Database::search_ring_id(&String::from("GLaDOs"));
        assert!(!found);
        // Insert new devices
        let row = Database::insert_new_device(&String::from("GLaDOs"), &String::from("PBody"), &String::from("Atlas"));
        assert!(row.is_ok());
        let row = Database::insert_new_device(&String::from("Tars"), &String::from(""), &String::from("alexa"));
        assert!(row.is_ok());
        // Search should succeed
        let found = Database::search_ring_id(&String::from("GLaDOs"));
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
        let row = Database::insert_new_device(&String::from("GLaDOs"), &String::from("PBody"), &String::from("Atlas"));
        assert!(row.is_ok());
        let row = Database::insert_new_device(&String::from("Tars"), &String::from(""), &String::from("alexa"));
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
        let row = Database::insert_new_device(&String::from("GLaDOs"), &String::from("PBody"), &String::from("Atlas"));
        assert!(row.is_ok());
        let row = Database::insert_new_device(&String::from("Tars"), &String::from(""), &String::from("alexa"));
        assert!(row.is_ok());
        // Update should update
        let row = Database::update_username(&String::from("GLaDOs"), &String::from("BPody"));
        assert!(row.is_ok());
        // Should be retrieven
        let device = Database::get_device(&String::from("GLaDOs"));
        assert!(device.0 == "GLaDOs");
        assert!(device.1 == "BPody");
        assert!(device.2 == "Atlas");
        teardown();
    }

    #[test]
    fn test_update_devicename() {
        setup();
        // Insert new devices
        let row = Database::insert_new_device(&String::from("GLaDOs"), &String::from("PBody"), &String::from("Atlas"));
        assert!(row.is_ok());
        let row = Database::insert_new_device(&String::from("Tars"), &String::from(""), &String::from("alexa"));
        assert!(row.is_ok());
        // Update should update
        let row = Database::update_devicename(&String::from("GLaDOs"), &String::from("Jupiter"));
        assert!(row.is_ok());
        // Should be retrieven
        let device = Database::get_device(&String::from("GLaDOs"));
        assert!(device.0 == "GLaDOs");
        assert!(device.1 == "PBody");
        assert!(device.2 == "Jupiter");
        teardown();
    }

    #[test]
    fn test_get_set_datatypes() {
        setup();
        // Invalid set, should get nothing
        let mut dts = Vec::new();
        dts.push(String::from("dt1"));
        dts.push(String::from("dt2"));
        let _ = Database::set_datatypes(&String::from("GLaDOs"), dts.clone());
        let get = Database::get_datatypes(&String::from("GLaDOs"));
        assert!(get.len() == 0);
        // Insert new device
        let row = Database::insert_new_device(&String::from("GLaDOs"), &String::from("PBody"), &String::from("Atlas"));
        assert!(row.is_ok());
        let get = Database::get_datatypes(&String::from("GLaDOs"));
        assert!(get.len() == 0);
        // Valid set should get what we set
        let _ = Database::set_datatypes(&String::from("GLaDOs"), dts.clone());
        let get = Database::get_datatypes(&String::from("GLaDOs"));
        assert!(get == dts);
        // Should be rewritable
        let _ = Database::set_datatypes(&String::from("GLaDOs"), Vec::new());
        let get = Database::get_datatypes(&String::from("GLaDOs"));
        assert!(get.len() == 0);
        teardown();
    }
}
