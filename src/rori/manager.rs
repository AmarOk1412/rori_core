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
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use time;

/**
 * This class is used to load RORI accounts and handle signals from Ring.
 * Should be one unique instance of this and is used to access the RORI server
 */
pub struct Manager {
    pub server: Server,

    ring_dbus: &'static str,
    configuration_path: &'static str,
    configuration_iface: &'static str,
}

impl Manager {
    /**
     * Init the RORI server, the database and retrieve the RING account linked
     * @param ring_id to retrieve
     * @return a Manager if success, else an error
     */
    pub fn init(ring_id: &str) -> Result<Manager, &'static str> {
        Database::init_db();
        let mut manager = Manager {
            server: Server::new(Account::null()),

            ring_dbus: "cx.ring.Ring",
            configuration_path: "/cx/ring/Ring/ConfigurationManager",
            configuration_iface: "cx.ring.Ring.ConfigurationManager",
        };
        manager.server.account = Manager::build_account(ring_id);
        if !manager.server.account.enabled {
            info!("{} was not enabled. Enable it", ring_id);
            manager.enable_account();
        }
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
    pub fn handle_signals(manager: Arc<Mutex<Manager>>, stop: Arc<AtomicBool>) {
        // Use another dbus connection to listen signals.
        let dbus_listener = Connection::get_private(BusType::Session).unwrap();
        dbus_listener.add_match("interface=cx.ring.Ring.ConfigurationManager,member=incomingAccountMessage").unwrap();
        dbus_listener.add_match("interface=cx.ring.Ring.ConfigurationManager,member=incomingTrustRequest").unwrap();
        dbus_listener.add_match("interface=cx.ring.Ring.ConfigurationManager,member=accountsChanged").unwrap(); // TODO
        dbus_listener.add_match("interface=cx.ring.Ring.ConfigurationManager,member=registrationStateChanged").unwrap(); // TODO
        // For each signals, call handlers.
        for i in dbus_listener.iter(100) {
            let mut m = manager.lock().unwrap();
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
                    // The device should send a new message to be registered to its RORI
                    m.server.add_new_anonymous_device(&from);
                }
            };
            if stop.load(Ordering::SeqCst) {
                break;
            }
        }
    }

    // Helpers

    /**
     * Add a RING account
     * @param main_info path or alias
     * @param password
     * @param from_archive if main_info is a path
     */
    pub fn add_account(main_info: &str, password: &str, from_archive: bool) {
        let mut details: HashMap<&str, &str> = HashMap::new();
        if from_archive {
            details.insert("Account.archivePath", main_info);
        } else {
            details.insert("Account.alias", main_info);
        }
        details.insert("Account.type", "RING");
        details.insert("Account.archivePassword", password);
        let details = Dict::new(details.iter());
        let dbus_msg = Message::new_method_call("cx.ring.Ring", "/cx/ring/Ring/ConfigurationManager",
                                                "cx.ring.Ring.ConfigurationManager",
                                                "addAccount")
                                                .ok().expect("addAccount fails. Please verify daemon's API.");
        let dbus = Connection::get_private(BusType::Session).ok().expect("dbus connection not ok");
        let response = dbus.send_with_reply_and_block(dbus_msg.append1(details), 2000).unwrap();
        // addAccount returns one argument, which is a string.
        let account_added: &str  = response.get1().unwrap_or("");
        info!("New account: {:?}", account_added);
    }

    /**
     * Get current ring accounts
     * @return current accounts
     */
    pub fn get_account_list() -> Vec<Account> {
        let mut account_list: Vec<Account> = Vec::new();
        let dbus_msg = Message::new_method_call("cx.ring.Ring", "/cx/ring/Ring/ConfigurationManager",
                                                "cx.ring.Ring.ConfigurationManager",
                                                "getAccountList")
                                                .ok().expect("getAccountList fails. Please verify daemon's API.");
        let dbus = Connection::get_private(BusType::Session).ok().expect("dbus connection not ok");
        let response = dbus.send_with_reply_and_block(dbus_msg, 2000).unwrap();
        // getAccountList returns one argument, which is an array of strings.
        let accounts: Array<&str, _>  = response.get1().unwrap();
        for account in accounts {
            account_list.push(Manager::build_account(account));
        }
        account_list
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
        let dbus_msg = Message::new_method_call(self.ring_dbus, self.configuration_path,
                                                self.configuration_iface,
                                                method).ok().expect("method call fails. Please verify daemon's API.");;
        let dbus = Connection::get_private(BusType::Session).ok().expect("connection not ok.");
        let response = dbus.send_with_reply_and_block(
            dbus_msg.append3(account_id, from, accept), 2000).unwrap();
        let result = response.get1().unwrap_or(false);
        info!("{} handles request from {} with success: {}", account_id, from, result);
        return result;
    }

    /**
     * Build a new account with an id from the daemon
     * @param id the account id to build
     * @return the account retrieven
     */
    fn build_account(id: &str) -> Account {
        let dbus_msg = Message::new_method_call("cx.ring.Ring", "/cx/ring/Ring/ConfigurationManager",
                                                "cx.ring.Ring.ConfigurationManager",
                                                "getAccountDetails").ok().expect("method call fails. Please verify daemon's API.");;
        let dbus = Connection::get_private(BusType::Session).ok().expect("connection not ok.");
        let response = dbus.send_with_reply_and_block(
                                           dbus_msg.append1(id), 2000
                                       ).ok().expect("Is the ring-daemon launched?");
        let details: Dict<&str, &str, _> = response.get1().unwrap();

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
     * Enable a Ring account
     * @param self
     */
    pub fn enable_account(&self) {
        let dbus_msg = Message::new_method_call(self.ring_dbus, self.configuration_path,
                                                self.configuration_iface,
                                                "sendRegister").ok().expect("method call fails. Please verify daemon's API.");;
        let dbus = Connection::get_private(BusType::Session).ok().expect("connection not ok.");
        let _ = dbus.send_with_reply_and_block(
            dbus_msg.append2(self.server.account.id.clone(), true), 2000);
    }

    /**
     * Retrievee all devices
     * @param self
     * @param account_id related
     * @return a Vec of devices ring_id
     */
    fn get_devices(&self, account_id: &str) -> Vec<String> {
        let mut devices: Vec<String> = Vec::new();
        let dbus_msg = Message::new_method_call(self.ring_dbus, self.configuration_path,
                                                self.configuration_iface,
                                                "getContacts").ok().expect("method call fails. Please verify daemon's API.");;
        let dbus = Connection::get_private(BusType::Session).ok().expect("connection not ok.");
        let response = dbus.send_with_reply_and_block(dbus_msg.append1(account_id), 2000).unwrap();
        let devices_vec: Array<Dict<&str, &str, _>, _> = response.get1().unwrap();
        for details in devices_vec {
            for detail in details {
                match detail {
                    (key, value) => {
                        if key == "id" {
                            devices.push(value.to_string());
                        }
                    }
                }
            }
        }
        devices
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
        let mut datatype = String::new();
        for detail in payloads.unwrap() {
            match detail {
                (key, value) => {
                    datatype = key.to_string();
                    body = value.to_string();
                }
            }
        };
        let interaction = Interaction {
            author_ring_id: author_ring_id,
            body: body,
            datatype: datatype,
            time: time::now()
        };
        Some((account_id.unwrap().to_string(), interaction))
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

    /**
     * Synchronizes contacts between database and daemon and init account.
     * @param self
     */
    fn load_contacts(&mut self) {
        let mut db_devices = Database::get_devices();
        let ring_devices = self.get_devices(&*self.server.account.id);

        // Remove non existing devices
        let mut idx: usize = 0;
        for device in db_devices.clone() {
            match ring_devices.iter().position(|c| c == &*device.0) {
                Some(_) => {
                    idx += 1;
                },
                None => {
                    info!("{} found in db but not from daemon, update db.", device.0);
                    db_devices.remove(idx);
                    let error_msg = format!("Failed to remove {} from database", device.0);
                    Database::remove_device(&device.0).ok().expect(&*error_msg);
                }
            }
        }

        // Add new devices
        for device in &ring_devices {
            match db_devices.iter().position(|c| &*c.0 == &*device) {
                Some(_) => {},
                None => {
                    info!("{} found from daemon but not in daemon, update db.", device);
                    db_devices.push((device.clone(), String::new(), String::new()));
                    let error_msg = format!("Failed to insert {} from database", device);
                    Database::insert_new_device(&device, &String::new(), &String::new()).ok().expect(&*error_msg);
                }
            }
        }

        self.server.load_devices(db_devices);
    }
}
