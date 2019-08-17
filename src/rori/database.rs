/**
 * Copyright (c) 2018, Sébastien Blin <sebastien.blin@enconn.fr>
 * All rights reserved.
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are met:
 *
 * * Redistributions of source code must retain the above copyright
 *  notice, this list of conditions and the following disclaimer.
 * * Redistributions in binary form must reproduce the above copyright
 *  notice, this list of conditions and the following disclaimer in the
 *  documentation and/or other materials provided with the distribution.
 * * Neither the name of the University of California, Berkeley nor the
 *  names of its contributors may be used to endorse or promote products
 *  derived from this software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE REGENTS AND CONTRIBUTORS ``AS IS'' AND ANY
 * EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
 * WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
 * DISCLAIMED. IN NO EVENT SHALL THE REGENTS AND CONTRIBUTORS BE LIABLE FOR ANY
 * DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
 * (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
 * LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND
 * ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
 * SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 **/

use rori::module::*;
use rusqlite;
use std::error::Error;
use string_error::static_err;

/**
 * This class furnish helpers to manipulate the rori.db sqlite database
 */
pub struct Database;

impl Database {
    /**
     * Set is_bridge to true
     * @param id of the device to modify
     * @return if success
     */
    pub fn bridgify(id: &i32) -> Result<usize, rusqlite::Error> {
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let mut stmt = conn.prepare("UPDATE devices SET is_bridge=1 WHERE id=:id").unwrap();
        stmt.execute_named(&[(":id", id)])
    }

    /**
     * Create tables in rori.db
     * NOTE: maybe has to change in case of migrations
     */
    pub fn init_db() {
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap_or(0);
        let mut do_migration = true;
        if version == 1 {
            do_migration = false;
        }
        if do_migration {
            info!("migrate database to version 1");
            conn.execute("CREATE TABLE IF NOT EXISTS devices (
                id               INTEGER PRIMARY KEY,
                hash             TEXT,
                username         TEXT,
                sub_author       TEXT,
                devicename       TEXT,
                additional_types TEXT,
                is_bridge        INTEGER
                )", rusqlite::NO_PARAMS).unwrap();
            conn.execute("CREATE TABLE IF NOT EXISTS modules (
                id          INTEGER PRIMARY KEY,
                name        TEXT,
                priority    INTEGER,
                enabled     BOOLEAN,
                type        TEXT,
                condition   TEXT,
                path        TEXT
                )", rusqlite::NO_PARAMS).unwrap();
            conn.execute("CREATE TABLE IF NOT EXISTS emotions (
                username    TEXT PRIMARY KEY,
                love        INTEGER,
                joy         INTEGER,
                surprise    INTEGER,
                anger       INTEGER,
                sadness     INTEGER,
                fear        INTEGER
                )", rusqlite::NO_PARAMS).unwrap();
            conn.pragma_update(None, "user_version", &1).unwrap();
        }
        info!("database ready");
    }

    pub fn is_bridge(hash: &String) -> bool {
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let mut stmt = conn.prepare("SELECT additional_types FROM devices WHERE hash=:hash AND is_bridge=1").unwrap();
        let mut rows = stmt.query_named(&[(":hash", hash)]).unwrap();
        if let Ok(Some(_)) = rows.next() {
            return true;
        }
        false
    }

    pub fn is_bridge_with_username(hash: &String, username: &String) -> bool {
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let mut stmt = conn.prepare("SELECT additional_types FROM devices WHERE hash=:hash AND username=:username AND is_bridge=1").unwrap();
        let mut rows = stmt.query_named(&[(":hash", hash), (":username", username)]).unwrap();
        if let Ok(Some(_)) = rows.next() {
            return true;
        }
        false
    }

    /**
     * get additional supported types for a device (text/plain is supported by default)
     * NOTE: because this is only used by rori_modules, don't have to save it on the rust side
     * @param id of the device
     * @return Vec<String> where each string is a supported datatype
     */
    pub fn get_datatypes(id: &i32) -> Vec<String> {
        let mut datatypes = Vec::new();
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let mut stmt = conn.prepare("SELECT additional_types FROM devices WHERE id=:id").unwrap();
        let mut rows = stmt.query_named(&[(":id", id)]).unwrap();
        if let Ok(Some(row)) = rows.next() {
            let row: String = row.get(0).unwrap_or(String::new());
            let dts: Vec<&str> = row.split(' ').collect();
            for dt in dts.into_iter() {
                if dt != "" {
                    datatypes.push(String::from(dt));
                }
            }
        }
        datatypes
    }

    /**
     * Get datatypes supported for the reception to execute modules conditions
     * @return Vec<String> where each string is a supported datatype
     */
    pub fn get_modules_datatypes() -> Vec<String> {
        let mut datatypes = Vec::new();
        datatypes.push(String::from("text/plain")); // Basic datatype handled by the core
        datatypes.push(String::from("rori/command")); // Basic datatype handled by the core
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let mut stmt = conn.prepare("SELECT DISTINCT type FROM modules;").unwrap();
        let mut rows = stmt.query(rusqlite::NO_PARAMS).unwrap();
        if let Ok(Some(row)) = rows.next() {
            let row: String = row.get(0).unwrap_or(String::new());
            datatypes.push(String::from(row));
        }
        datatypes
    }

    /**
     * Insert new device
     * @param hash the ring id of this device
     * @param username username linked
     * @param devicename device's name related
     * @return the line's id inserted if success, else an error
     */
    pub fn insert_new_device(hash: &String, username: &String, devicename: &String, is_bridge: bool) -> Result<usize, Box<dyn Error>> {
        let conn = rusqlite::Connection::open("rori.db").unwrap();

        // If already exists
        if is_bridge {
            let mut stmt = conn.prepare("SELECT id FROM devices WHERE hash=:hash AND username=:username AND devicename=:devicename").unwrap();
            let mut rows = stmt.query_named(&[(":hash", hash), (":username", username), (":devicename", devicename)]).unwrap();
            while let Ok(Some(_)) = rows.next() {
                return Err(static_err("Device already inserted"));
            }
        } else {
            let mut stmt = conn.prepare("SELECT id FROM devices WHERE hash=:hash").unwrap();
            let mut rows = stmt.query_named(&[(":hash", hash)]).unwrap();
            while let Ok(Some(_)) = rows.next() {
                return Err(static_err("Device already inserted"));
            }
        }

        // Else insert!
        let mut conn = conn.prepare("INSERT INTO devices (hash, username, sub_author, devicename, additional_types, is_bridge)
                                     VALUES (:hash, :username, \"\", :devicename, \"\", :is_bridge)").unwrap();
        match conn.execute_named(&[(":hash", hash), (":username", username), (":devicename", devicename), (":is_bridge", &is_bridge)]) {
            Ok(_) => {
                return Ok(Database::get_device(hash, username).0 as usize);
            }
            Err(e) => {
                return Err(Box::new(e));
            }
        }
    }

    /**
     * Get enabled modules for a priority
     * @param priority
     * @return a vector of modules
     */
    pub fn get_enabled_modules(priority: u64) -> Vec<Module> {
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let mut stmt = conn.prepare("SELECT name, condition, path \
                                     FROM modules WHERE priority=:priority AND enabled=1"
                                   ).unwrap();
       let mut rows = stmt.query_named(&[(":priority", &priority.to_string())]).unwrap();
       let mut modules = Vec::new();
       while let Ok(Some(row)) = rows.next() {
           modules.push(
               Module {
                   condition: Box::new(TextCondition::new(row.get(1).unwrap_or(String::new()))),
                   name: row.get(0).unwrap_or(String::new()),
                   path: row.get(2).unwrap_or(String::new()),
                   priority: priority,
                   enabled: true,
               }
           );
       }
       modules
    }

    /**
     * Return one device
     * @hash the ring id of the device to search
     * @return (id, hash, username, devicename, is_bridge) or empty strings with id = -1 if hash not found
     */
    pub fn get_device(hash: &String, username: &String) -> (i32, String, String, String, i32) {
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let mut stmt = conn.prepare("SELECT id, hash, username, devicename, is_bridge FROM devices \
            WHERE hash=:hash AND username=:username").unwrap();
        let mut rows = stmt.query_named(&[(":hash", hash), (":username", username)]).unwrap();
        while let Ok(Some(row)) = rows.next() {
            return (row.get(0).unwrap_or(0), row.get(1).unwrap_or(String::new()), row.get(2).unwrap_or(String::new()), row.get(3).unwrap_or(String::new()), row.get(4).unwrap_or(0));
        }
        (-1, String::new(), String::new(), String::new(), 0)
    }

    /**
     * Return all devices
     * @return a Vector of devices (id, hash, username, devicename, is_bridge)
     */
    pub fn get_devices() -> Vec<(i32, String, String, String, bool)> {
        let mut devices: Vec<(i32, String, String, String, bool)> = Vec::new();
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let mut stmt = conn.prepare("SELECT id, hash, username, devicename, is_bridge FROM devices").unwrap();
        let mut rows = stmt.query(rusqlite::NO_PARAMS).unwrap();
        while let Ok(Some(row)) = rows.next() {
            devices.push((row.get(0).unwrap_or(0), row.get(1).unwrap_or(String::new()), row.get(2).unwrap_or(String::new()), row.get(3).unwrap_or(String::new()), row.get(4).unwrap_or(false)));
        }
        devices
    }

    /**
     * @note till Rust doesn't supports optional parameters
     * Return all devices for a hash
     * @return a Vector of devices (id, hash, username, devicename, is_bridge)
     */
    pub fn get_devices_for_hash(hash: &str) -> Vec<(i32, String, String, String, bool)> {
        let mut devices: Vec<(i32, String, String, String, bool)> = Vec::new();
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let mut stmt = conn.prepare("SELECT id, hash, username, devicename, is_bridge FROM devices \
            WHERE hash=:hash").unwrap();
        let mut rows = stmt.query_named(&[(":hash", &hash.to_string())]).unwrap();
        while let Ok(Some(row)) = rows.next() {
            devices.push((row.get(0).unwrap_or(0), row.get(1).unwrap_or(String::new()), row.get(2).unwrap_or(String::new()), row.get(3).unwrap_or(String::new()), row.get(4).unwrap_or(false)));
        }
        devices
    }

    /**
     * @note till Rust doesn't supports optional parameters
     * Return all devices for an username
     * @return a Vector of devices (id, hash, username, devicename, is_bridge)
     */
    pub fn get_devices_for_username(username: &str) -> Vec<(i32, String, String, String, bool)> {
        let mut devices: Vec<(i32, String, String, String, bool)> = Vec::new();
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let mut stmt = conn.prepare("SELECT id, hash, username, devicename, is_bridge FROM devices \
            WHERE username=:username").unwrap();
        let mut rows = stmt.query_named(&[(":username", &username.to_string())]).unwrap();
        while let Ok(Some(row)) = rows.next() {
            devices.push((row.get(0).unwrap_or(0), row.get(1).unwrap_or(String::new()), row.get(2).unwrap_or(String::new()), row.get(3).unwrap_or(String::new()), row.get(4).unwrap_or(false)));
        }
        devices
    }

    /**
     * Return the last priority to treat
     * @return i64
     */
    pub fn get_descending_priorities() -> Vec<i64> {
        let mut result = Vec::new();
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let mut stmt = conn.prepare("SELECT DISTINCT priority FROM modules ORDER BY priority ASC").unwrap();
        let mut rows = stmt.query(rusqlite::NO_PARAMS).unwrap();
        while let Ok(Some(row)) = rows.next() {
            result.push(row.get(0).unwrap_or(0));
        }
        result
    }

    /**
     * Remove a device from the devices table
     * @param hash to remove
     * @return the id of the removed row or an error
     */
    pub fn remove_device(id: &i32) -> Result<usize, rusqlite::Error> {
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let mut conn = conn.prepare("DELETE FROM devices WHERE id=:id").unwrap();
        conn.execute_named(&[(":id", id)])
    }

    /**
     * Search a devicename
     * NOTE: search a full devicename. So, username_devicename
     * @param username related to search
     * @param devicename to search
     * @return if found
     */
    pub fn search_devicename(username: &String, devicename: &String) -> bool {
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let mut stmt = conn.prepare("SELECT * FROM devices WHERE username=:username AND devicename=:devicename").unwrap();
        let mut rows = stmt.query_named(&[(":username", username), (":devicename", devicename)]).unwrap();
        while let Ok(Some(_)) = rows.next() {
            return true;
        }
        false
    }

    /**
     * Search a hash
     * @param hash to search
     * @return if found
     */
    pub fn search_hash(hash: &String) -> bool {
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let mut stmt = conn.prepare("SELECT * FROM devices WHERE hash=:hash").unwrap();
        let mut rows = stmt.query_named(&[(":hash", hash)]).unwrap();
        while let Ok(Some(_)) = rows.next() {
            return true;
        }
        false
    }

    /**
     * Search a username
     * @param username to search
     * @return if found
     */
    pub fn search_username(username: &String) -> bool {
        if username.to_lowercase() == "rori" {
            // RESERVED
            return true;
        }
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let mut stmt = conn.prepare("SELECT * FROM devices WHERE username=:username").unwrap();
        let mut rows = stmt.query_named(&[(":username", username)]).unwrap();
        while let Ok(Some(_)) = rows.next() {
            return true;
        }
        false
    }

    /**
     * Set additional supported types for a device
     * @param id of the device to modify
     * @param datatypes to set
     * @return if success
     */
    pub fn set_datatypes(id: &i32, datatypes: Vec<String>) -> Result<usize, rusqlite::Error> {
        let datatypes = datatypes.join(" ");
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let mut stmt = conn.prepare("UPDATE devices SET additional_types=:additional_types WHERE id=:id").unwrap();
        stmt.execute_named(&[(":id", id), (":additional_types", &String::from(datatypes))])
    }

    /**
     * get sub_author via it's id
     * @param hash of the device
     * @param sub_author of the device
     * @return String linked
     */
    pub fn sub_author(hash: &String, sub_author: &String) -> String {
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let mut stmt = conn.prepare("SELECT username FROM devices WHERE hash=:hash AND sub_author=:sub_author").unwrap();
        let mut rows = stmt.query_named(&[(":hash", hash), (":sub_author", sub_author)]).unwrap();
        if let Ok(Some(row)) = rows.next() {
            let username : String = row.get(0).unwrap_or(String::new());
            return username;
        }
        String::new()
    }

    /**
     * get sub_author_id via it's userne
     * @param hash of the device
     * @param sub_author of the device
     * @return String linked
     */
    pub fn sub_author_id(hash: &String, username: &String) -> String {
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let mut stmt = conn.prepare("SELECT sub_author FROM devices WHERE hash=:hash AND username=:username").unwrap();
        let mut rows = stmt.query_named(&[(":hash", hash), (":username", username)]).unwrap();
        if let Ok(Some(row)) = rows.next() {
            let sub_author : String = row.get(0).unwrap_or(String::new());
            return sub_author;
        }
        String::new()
    }


    /**
     * Update a devicename
     * @param id to search
     * @param devicename new devicename
     * @return the id of the modified row if success else an error
     */
    pub fn update_devicename(id: &i32, devicename: &String) -> Result<usize, rusqlite::Error> {
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let mut stmt = conn.prepare("UPDATE devices SET devicename=:devicename WHERE id=:id").unwrap();
        stmt.execute_named(&[(":id", id), (":devicename", devicename)])
    }

    /**
     * Set sub_author for a device
     * @param id of the device to modify
     * @param sub_author to set
     * @return if success
     */
    pub fn update_sub_author(id: &i32, sub_author: &String) -> Result<usize, rusqlite::Error> {
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let mut stmt = conn.prepare("UPDATE devices SET sub_author=:sub_author WHERE id=:id").unwrap();
        stmt.execute_named(&[(":id", id), (":sub_author", sub_author)])
    }

    /**
     * Update a username
     * @param id to search
     * @param username new username
     * @return the id of the modified row if success else an error
     */
    pub fn update_username(id: &i32, username: &String) -> Result<usize, rusqlite::Error> {
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let mut stmt = conn.prepare("UPDATE devices SET username=:username WHERE id=:id").unwrap();
        stmt.execute_named(&[(":id", id), (":username", username)])
    }
}
