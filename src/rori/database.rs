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
