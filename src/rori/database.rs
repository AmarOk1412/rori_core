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

use rusqlite;

// TODO trait
pub struct Database {

}

impl Database {
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
            conn.execute("CREATE TABLE users (
                ring_id     TEXT PRIMARY KEY,
                username    TEXT,
                devicename  TEXT
                )", &[]).unwrap();
            conn.execute("PRAGMA user_version = 1", &[]).unwrap();
        }
        info!("database ready");
    }

    pub fn insert_new_user(ring_id: &String, username: &String, devicename: &String) -> Result<i32, rusqlite::Error> {
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let mut conn = conn.prepare("INSERT INTO users (ring_id, username, devicename)
                                     VALUES (:ring_id, :username, :devicename)").unwrap();
        conn.execute_named(&[(":ring_id", ring_id), (":username", username), (":devicename", devicename)])
    }

    pub fn get_users() -> Vec<(String, String, String)> {
        let mut users: Vec<(String, String, String)> = Vec::new();
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let mut stmt = conn.prepare("SELECT ring_id, username, devicename  FROM users").unwrap();
        let mut rows = stmt.query(&[]).unwrap();
        while let Some(row) = rows.next() {
            let row = row.unwrap();
            users.push((row.get(0), row.get(1), row.get(2)));
        }
        users
    }

    pub fn remove_user(ring_id: &String) -> Result<i32, rusqlite::Error> {
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let mut conn = conn.prepare("DELETE FROM users WHERE ring_id=:ring_id").unwrap();
        conn.execute_named(&[(":ring_id", ring_id)])
    }

    pub fn search_username(username: &String) -> bool {
        if username == "rori" {
            return true;
        }
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let mut stmt = conn.prepare("SELECT * FROM users WHERE username=:username").unwrap();
        let mut rows = stmt.query_named(&[(":username", username)]).unwrap();
        while let Some(_) = rows.next() {
            return true;
        }
        false
    }

    pub fn search_devicename(username: &String, devicename: &String) -> bool {
        if username == "rori" {
            return true;
        }
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let mut stmt = conn.prepare("SELECT * FROM users WHERE username=:username AND devicename=:devicename").unwrap();
        let mut rows = stmt.query_named(&[(":username", username), (":devicename", devicename)]).unwrap();
        while let Some(_) = rows.next() {
            return true;
        }
        false
    }

    pub fn update_username(ring_id: &String, username: &String) -> Result<i32, rusqlite::Error> {
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let mut stmt = conn.prepare("UPDATE users SET username=:username WHERE ring_id=:ring_id").unwrap();
        stmt.execute_named(&[(":ring_id", ring_id), (":username", username)])
    }

    pub fn update_devicename(ring_id: &String, devicename: &String) -> Result<i32, rusqlite::Error> {
        let conn = rusqlite::Connection::open("rori.db").unwrap();
        let mut stmt = conn.prepare("UPDATE users SET devicename=:devicename WHERE ring_id=:ring_id").unwrap();
        stmt.execute_named(&[(":ring_id", ring_id), (":devicename", devicename)])
    }
}
