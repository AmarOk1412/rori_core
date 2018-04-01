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
use std::sync::{Arc, Mutex};

/**
 * This class furnish helpers to manipulate the rori.db sqlite database
 */
pub struct Database;

impl Database {
    /**
     * Create tables in rori.db
     * NOTE: maybe has to change in case of migrations
     */
    pub fn init_db() {
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let mut q = conn.prepare("PRAGMA user_version").unwrap();
        let version: i64 = q.query_row(&[], |row| row.get(0)).unwrap_or(0);
        let mut do_migration = true;
        if version == 1 {
            do_migration = false;
        }
        if do_migration {
            info!("migrate database to version 1");
            conn.execute("CREATE TABLE devices (
                ring_id     TEXT PRIMARY KEY,
                username    TEXT,
                devicename  TEXT
                )", &[]).unwrap();
            conn.execute("CREATE TABLE modules (
                id          INTEGER PRIMARY KEY,
                name        TEXT,
                priority    INTEGER,
                enabled     BOOLEAN,
                type        TEXT,
                condition   TEXT,
                path        TEXT,
                metadatas   TEXT
                )", &[]).unwrap();
            conn.execute("PRAGMA user_version = 1", &[]).unwrap();
        }
        info!("database ready");
    }

    /**
     * Insert new device
     * @param ring_id the ring id of this device
     * @param username username linked
     * @param devicename device's name related
     * @return the line's id inserted if success, else an error
     */
    pub fn insert_new_device(ring_id: &String, username: &String, devicename: &String) -> Result<i32, rusqlite::Error> {
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let mut conn = conn.prepare("INSERT INTO devices (ring_id, username, devicename)
                                     VALUES (:ring_id, :username, :devicename)").unwrap();
        conn.execute_named(&[(":ring_id", ring_id), (":username", username), (":devicename", devicename)])
    }

    pub fn get_enabled_modules(priority: u64) -> Vec<Module> {
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let mut stmt = conn.prepare("SELECT name, condition, path \
                                     FROM modules WHERE priority=:priority AND enabled=1"
                                   ).unwrap();
       let mut rows = stmt.query_named(&[(":priority", &priority.to_string())]).unwrap();
       let mut modules = Vec::new();
       while let Some(row) = rows.next() {
           let row = row.unwrap();
           modules.push(
               Module {
                   condition: Box::new(TextCondition::new(row.get(1))),
                   name: row.get(0),
                   path: row.get(2),
                   priority: priority,
                   enabled: true,
               }
           );
       }
       modules
    }

    /**
     * Return one device
     * @ring_id the ring id of the device to search
     * @return (ring_id, username, devicename) or empty strings if ring_id not found
     */
    pub fn get_device(ring_id: &String) -> (String, String, String) {
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let mut stmt = conn.prepare("SELECT ring_id, username, devicename FROM devices WHERE ring_id=:ring_id").unwrap();
        let mut rows = stmt.query_named(&[(":ring_id", ring_id)]).unwrap();
        while let Some(row) = rows.next() {
            let row = row.unwrap();
            return (row.get(0), row.get(1), row.get(2));
        }
        (String::new(), String::new(), String::new())
    }

    /**
     * Return all devices
     * @return a Vector of devices (ring_id, username, devicename)
     */
    pub fn get_devices() -> Vec<(String, String, String)> {
        let mut devices: Vec<(String, String, String)> = Vec::new();
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let mut stmt = conn.prepare("SELECT ring_id, username, devicename FROM devices").unwrap();
        let mut rows = stmt.query(&[]).unwrap();
        while let Some(row) = rows.next() {
            let row = row.unwrap();
            devices.push((row.get(0), row.get(1), row.get(2)));
        }
        devices
    }

    pub fn get_max_priority() -> i64 {
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let mut stmt = conn.prepare("SELECT MAX(priority) FROM modules").unwrap();
        let mut rows = stmt.query(&[]).unwrap();
        while let Some(row) = rows.next() {
            let row = row.unwrap();
            return row.get(0);
        }
        0
    }

    /**
     * Remove a device from the devices table
     * @param ring_id to remove
     * @return the id of the removed row or an error
     */
    pub fn remove_device(ring_id: &String) -> Result<i32, rusqlite::Error> {
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let mut conn = conn.prepare("DELETE FROM devices WHERE ring_id=:ring_id").unwrap();
        conn.execute_named(&[(":ring_id", ring_id)])
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
        while let Some(_) = rows.next() {
            return true;
        }
        false
    }

    /**
     * Search a ring_id
     * @param ring_id to search
     * @return if found
     */
    pub fn search_ring_id(ring_id: &String) -> bool {
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let mut stmt = conn.prepare("SELECT * FROM devices WHERE ring_id=:ring_id").unwrap();
        let mut rows = stmt.query_named(&[(":ring_id", ring_id)]).unwrap();
        while let Some(_) = rows.next() {
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
        while let Some(_) = rows.next() {
            return true;
        }
        false
    }

    /**
     * Update a devicename
     * @param ring_id to search
     * @param devicename new devicename
     * @return the id of the modified row if success else an error
     */
    pub fn update_devicename(ring_id: &String, devicename: &String) -> Result<i32, rusqlite::Error> {
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let mut stmt = conn.prepare("UPDATE devices SET devicename=:devicename WHERE ring_id=:ring_id").unwrap();
        stmt.execute_named(&[(":ring_id", ring_id), (":devicename", devicename)])
    }

    /**
     * Update a username
     * @param ring_id to search
     * @param username new username
     * @return the id of the modified row if success else an error
     */
    pub fn update_username(ring_id: &String, username: &String) -> Result<i32, rusqlite::Error> {
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let mut stmt = conn.prepare("UPDATE devices SET username=:username WHERE ring_id=:ring_id").unwrap();
        stmt.execute_named(&[(":ring_id", ring_id), (":username", username)])
    }
}
