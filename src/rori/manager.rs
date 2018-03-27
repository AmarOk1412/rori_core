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

use dbus::{Connection, ConnectionItem, BusType, Message};
use dbus::arg::{Array, Dict};
use rori::account::Account;
use rori::database::Database;
use rori::interaction::Interaction;
use rori::server::Server;
use std::sync::{Arc, Mutex};
use time;

/**
 * This class is used to load RORI accounts and handle signals from Ring.
 * Should be one unique instance of this and is used to access accounts informations
 */
pub struct Manager {
    pub server: Server,

    ring_dbus: &'static str,
    configuration_path: &'static str,
    configuration_iface: &'static str,
}

impl Manager {
    pub fn init(ring_id: &str) -> Result<Manager, &'static str> {
        Database::init_db();
        let mut manager = Manager {
            server: Server::new(Account::null()),

            ring_dbus: "cx.ring.Ring",
            configuration_path: "/cx/ring/Ring/ConfigurationManager",
            configuration_iface: "cx.ring.Ring.ConfigurationManager",
        };
        manager.server.account = manager.build_account(ring_id);
        debug!("Get: {}", manager.server.account.ring_id);
        if manager.server.account.ring_id == "" {
            return Err("Cannot build RORI account, please check configuration");
        }
        manager.load_contacts();
        info!("{}: Account loaded", manager.server.account.id);
        Ok(manager)
    }

    /**
     * Listen from interresting signals from dbus and call handlers
     * @param self
     */
    pub fn handle_signals(manager: Arc<Mutex<Manager>>) {
        // Use another dbus connection to listen signals.
        let dbus_listener = Connection::get_private(BusType::Session).unwrap();
        dbus_listener.add_match("interface=cx.ring.Ring.ConfigurationManager,member=incomingAccountMessage").unwrap();
        dbus_listener.add_match("interface=cx.ring.Ring.ConfigurationManager,member=incomingTrustRequest").unwrap();
        dbus_listener.add_match("interface=cx.ring.Ring.ConfigurationManager,member=accountsChanged").unwrap();
        dbus_listener.add_match("interface=cx.ring.Ring.ConfigurationManager,member=registrationStateChanged").unwrap();
        // For each signals, call handlers.
        for i in dbus_listener.iter(1) {
            let mut m = manager.lock().unwrap();
            m.handle_accounts_signals(&i);
            m.handle_registration_changed(&i);
            if let Some((account_id, interaction)) = m.handle_interactions(&i) {
                if account_id == m.server.account.id {
                    info!("New interaction for {}: {}", account_id, interaction);
                    // NOTE: if new ring_id, should be added to anonymouses
                    m.server.handle_interaction(interaction);
                }
            };
            if let Some((account_id, from)) = m.handle_requests(&i) {
                if account_id == m.server.account.id {
                    info!("New request from {}", from);
                    m.accept_request(&*account_id, &*from, true);
                    // At first, the new account is considered as anonymous
                    // The user should send a new message to be registered to its RORI
                    m.server.add_new_anonymous_user(&from);
                }
            };
        }
    }

// Private stuff

    /**
     * Accept a trust request from somebody
     * @param self
     * @param account_id the account who accepts the request
     * @param from the contact to accept
     * @param accept true if accept the request
     * @return if the contact was accepted
     */
    fn accept_request(&self, account_id: &str, from: &str, accept: bool) -> bool {
        let method = if accept {"acceptTrustRequest"} else {"discardTrustRequest"};
        let dbus_msg = Message::new_method_call(self.ring_dbus, self.configuration_path, self.configuration_iface,
                                                method);
        if !dbus_msg.is_ok() {
            error!("method call fails. Please verify daemon's API.");
            return false;
        }
        let conn = Connection::get_private(BusType::Session);
        if !conn.is_ok() {
            error!("connection not ok.");
            return false;
        }
        let dbus = conn.unwrap();
        let response = dbus.send_with_reply_and_block(
            dbus_msg.unwrap().append3(account_id, from, accept), 2000).unwrap();
        match response.get1() {
            Some(result) => {
                info!("{} handles request from {} with success", account_id, from);
                return result;
            },
            None => {
                warn!("{} handles request from {} with failure", account_id, from);
                return false;
            }
        };
    }

    /**
     * Build a new account with an id from the daemon
     * @param self
     * @param id the account id to build
     * @return the account retrieven
     */
    fn build_account(&self, id: &str) -> Account {
        let dbus_msg = Message::new_method_call(self.ring_dbus, self.configuration_path,
                                                self.configuration_iface,
                                                "getAccountDetails");
        if !dbus_msg.is_ok() {
            error!("getAccountDetails fails. Please verify daemon's API.");
            return Account::null();
        }
        let conn = Connection::get_private(BusType::Session);
        if !conn.is_ok() {
            error!("connection not ok.");
            return Account::null();
        }
        let dbus = conn.unwrap();
        let response = dbus.send_with_reply_and_block(
                                           dbus_msg.unwrap().append1(id), 2000
                                       ).unwrap();
        let details: Dict<&str, &str, _> = match response.get1() {
            Some(details) => details,
            None => {
                return Account::null();
            }
        };

        let mut account = Account::null();
        account.id = id.to_owned();
        for detail in details {
            match detail {
                (key, value) => {
                    if key == "Account.enable" {
                        account.enabled = value == "true";
                    }
                    if key == "Account.alias" {
                        account.alias = String::from(value);
                    }
                    if key == "Account.username" {
                        account.ring_id = String::from(value);
                    }
                }
            }
        }
        account
    }

    /**
     * Update current RORI account by handling accountsChanged signals from daemon.
     * @param self
     * @param ci
     */
    fn handle_accounts_signals(&mut self, ci: &ConnectionItem) {
        // Check signal
        let msg = if let &ConnectionItem::Signal(ref signal) = ci { signal } else { return };
        if &*msg.interface().unwrap() != "cx.ring.Ring.ConfigurationManager" { return };
        if &*msg.member().unwrap() != "accountsChanged" { return };
        // TODO test if RORI accounts is still enabled + still exists
    }

    /**
    * Handle new interactions signals
    * @param self
    * @param ci
    * @return (accountId, interaction)
    */
    fn handle_interactions(&self, ci: &ConnectionItem) -> Option<(String, Interaction)> {
        // Check signal
        let msg = if let &ConnectionItem::Signal(ref signal) = ci { signal } else { return None };
        if &*msg.interface().unwrap() != "cx.ring.Ring.ConfigurationManager" { return None };
        if &*msg.member().unwrap() != "incomingAccountMessage" { return None };
        // incomingAccountMessage return three arguments
        let (account_id, author_ring_id, payloads) = msg.get3::<&str, &str, Dict<&str, &str, _>>();
        let author_ring_id = author_ring_id.unwrap().to_string();
        let mut body = String::new();
        for detail in payloads.unwrap() {
            // TODO handle other interactions
            match detail {
                (key, value) => {
                    if key == "text/plain" {
                        body = value.to_string();
                    }
                }
            }
        };
        let interaction = Interaction {
            author_ring_id: author_ring_id,
            body: body,
            time: time::now()
        };
        Some((account_id.unwrap().to_string(), interaction))
    }

    /**
     * Update current RORI account by handling accountsChanged signals from daemon
     * @param self
     * @param ci
     */
    fn handle_registration_changed(&self, ci: &ConnectionItem) {
        // Check signal
        let msg = if let &ConnectionItem::Signal(ref signal) = ci { signal } else { return };
        if &*msg.interface().unwrap() != "cx.ring.Ring.ConfigurationManager" { return };
        if &*msg.member().unwrap() != "registrationStateChanged" { return };
        // let (account_id, registration_state, _, _) = msg.get4::<&str, &str, u64, &str>();
        // TODO the account can be disabled. Inform UI
    }

    /**
     * Handle new pending requests signals
     * @param self
     * @param ci
     * @return (accountId, from)
     */
    fn handle_requests(&self, ci: &ConnectionItem) -> Option<(String, String)> {
        // Check signal
        let msg = if let &ConnectionItem::Signal(ref signal) = ci { signal } else { return None };
        if &*msg.interface().unwrap() != "cx.ring.Ring.ConfigurationManager" { return None };
        if &*msg.member().unwrap() != "incomingTrustRequest" { return None };
        // incomingTrustRequest return three arguments
        let (account_id, from, _, _) = msg.get4::<&str, &str, Dict<&str, &str, _>, u64>();
        Some((account_id.unwrap().to_string(), from.unwrap().to_string()))
    }


    fn get_users(&self, account_id: &str) -> Vec<String> {
        let mut users: Vec<String> = Vec::new();
        let dbus_msg = Message::new_method_call(self.ring_dbus, self.configuration_path, self.configuration_iface,
                                                "getContacts");
        if !dbus_msg.is_ok() {
            error!("getContacts fails. Please verify daemon's API.");
            return users;
        }
        let conn = Connection::get_private(BusType::Session);
        if !conn.is_ok() {
            return Vec::new();
        }
        let dbus = conn.unwrap();
        let response = dbus.send_with_reply_and_block(dbus_msg.unwrap().append1(account_id), 2000).unwrap();
        let users_vec: Array<Dict<&str, &str, _>, _> = match response.get1() {
            Some(details) => details,
            None => {
                return users;
            }
        };
        for details in users_vec {
            for detail in details {
                match detail {
                    (key, value) => {
                        if key == "id" {
                            users.push(value.to_string());
                        }
                    }
                }
            }
        }
        users
    }


    /**
     * Synchronizes contacts between database and daemon and init account.
     * @param self
     */
    fn load_contacts(&mut self) {
        let mut db_users = Database::get_users();
        let ring_users = self.get_users(&*self.server.account.id);

        // Remove non existing users
        let mut idx: usize = 0;
        for user in db_users.clone() {
            match ring_users.iter().position(|c| c == &*user.0) {
                Some(_) => {
                    idx += 1;
                },
                None => {
                    info!("{} found in db but not from daemon, update db.", user.0);
                    db_users.remove(idx);
                    match Database::remove_user(&user.0) {
                        Ok(_) => {}
                        _ => {
                            error!("Failed to remove {} from database", user.0);
                        }
                    }
                }
            }
        }

        // Add new users
        for user in &ring_users {
            match db_users.iter().position(|c| &*c.0 == &*user) {
                Some(_) => {},
                None => {
                    info!("{} found from daemon but not in daemon, update db.", user);
                    db_users.push((user.clone(), String::new(), String::new()));
                    match Database::insert_new_user(&user, &String::new(), &String::new()) {
                        Ok(_) => {}
                        _ => {
                            error!("Failed to insert {} from database", user);
                        }
                    }
                }
            }
        }

        self.server.load_users(db_users);
    }
}
