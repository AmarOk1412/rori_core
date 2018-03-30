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

use dbus::{Connection, BusType, Message};
use dbus::arg::Dict;
use rori::database::Database;
use rori::user::{Device, User};
use rori::interaction::Interaction;
use rori::account::Account;
use std::collections::HashMap;

/**
 * Core class.
 * Used to manages users and handle interactions with these users
 */
#[derive(Debug, Clone)]
pub struct Server {
    pub registered_users: Vec<User>,
    pub anonymous_user: User,
    pub account: Account,

    ring_dbus: &'static str,
    configuration_path: &'static str,
    configuration_iface: &'static str,
    id_to_account_linker: Vec<(String, String, bool)>
}

impl Server {
    /**
     * Generate a new Server with no devices. Devices must be loaded with load_devices()
     */
    pub fn new(account: Account) -> Server {
        Server {
            registered_users: Vec::new(),
            anonymous_user: User::new(),
            account: account,

            ring_dbus: "cx.ring.Ring",
            configuration_path: "/cx/ring/Ring/ConfigurationManager",
            configuration_iface: "cx.ring.Ring.ConfigurationManager",
            id_to_account_linker: Vec::new()
        }
    }

    /**
     * Add new device for the anonymous user
     * @param self
     * @param ring_id device to add
     */
    pub fn add_new_anonymous_device(&mut self, ring_id: &String) -> bool {
        let insert_into_db = Database::insert_new_device(ring_id, &String::new(), &String::new());
        match insert_into_db {
            Ok(_) => {}
            _ => {
                error!("add_new_anonymous_device failed");
                return false;
            }
        }
        self.anonymous_user.devices.push(Device::new(&ring_id));
        info!("{} added to anonymouses", ring_id);
        true
    }

    /**
     * Retrieve a ring_id for a given username or devicename
     * @param self
     * @param name username or devicename to find
     * @return the ring_id if found, else an empty String
     */
    pub fn get_ring_id(&mut self, name: &String) -> String {
        if name.to_lowercase() == "rori" {
            return self.account.ring_id.clone();
        }
        for mut registered in &self.registered_users {
            // Search if username match
            if &*registered.name == name {
                return registered.devices.first().unwrap().ring_id.clone();
            }
            // Search if devicename match
            for device in &registered.devices {
                if name == &*format!("{}_{}", registered.name, &*device.name) {
                    return device.ring_id.clone();
                }
            }
        }
        return String::new();
    }

    /**
     * Handle new interaction from Manager
     * @param self
     * @param interaction to process
     */
    pub fn handle_interaction(&mut self, interaction: Interaction) {
        // Find linked device
        let mut username = String::new();
        let mut user_found = false;
        for device in &self.anonymous_user.devices {
            if &*interaction.author_ring_id == &*device.ring_id {
                user_found = true;
                break;
            }
        }
        if !user_found {
            // User not found, continue to search.
            for mut registered in &self.registered_users {
                for device in &registered.devices {
                    if &*interaction.author_ring_id == &*device.ring_id {
                        user_found = true;
                        username = registered.name.clone();
                        break;
                    }
                }
            }
        }

        if !user_found {
            // User not found add it
            self.add_contact(&*self.account.id, &*interaction.author_ring_id);
            self.add_new_anonymous_device(&interaction.author_ring_id);
        }

        // TODO: link to old RORI structures to process messages
        // TODO should be handle by a module
        if username.len() == 0 {
            // Anonymous to user
            if interaction.body.starts_with("/register") {
                let split: Vec<&str> = interaction.body.split(' ').collect();
                if split.len() < 2 {
                    warn!("register received, but no username detected");
                    return;
                }
                self.try_register_username(&interaction.author_ring_id,
                                           &String::from(*split.get(1).unwrap()));
            }
        } else {
            if interaction.body.starts_with("/add_device") {
                // User wants to register a device
                // /add_device name (ring_id)
               let split: Vec<&str> = interaction.body.split(' ').collect();
               if split.len() < 2 {
                   warn!("add_device received, but no device detected");
                   return;
               }
               let mut device_to_add = interaction.author_ring_id.clone();
               if split.len() == 3 && split.last().unwrap_or(&"").len() > 0 {
                   device_to_add = split.last().unwrap_or(&"").to_string();
               }
               self.try_register_device(&interaction.author_ring_id, &device_to_add, &username,
                                        &String::from(*split.get(1).unwrap()));
            } else if interaction.body.starts_with("/rm_device") {
                // User wants to revoke a device
                // /rm_device name (ring_id)
                let mut device_to_remove = interaction.author_ring_id.clone();
                let split: Vec<&str> = interaction.body.split(' ').collect();
                if split.len() == 2 && split.last().unwrap_or(&"").len() > 0 {
                    device_to_remove = split.last().unwrap_or(&"").to_string();
                }
                self.try_remove_device(&interaction.author_ring_id, &device_to_remove);
            } else if interaction.body.starts_with("/unregister") {
                // User wants to unregister
                self.try_unregister(&interaction.author_ring_id);
            }
        }
        // Handle multi-devices
        if interaction.body.starts_with("/link") {
            let split: Vec<&str> = interaction.body.split(' ').collect();
            if split.len() < 2 {
                warn!("link received, but no argument detected");
                return;
            }
            self.try_link_new_device(&interaction.author_ring_id,
                                     &String::from(*split.get(1).unwrap()));
        }
    }

    /**
     * Build users from given devices
     * NOTE: should be in database.
     * @param self
     * @param devices to process
     */
    pub fn load_devices(&mut self, devices: Vec<(String, String, String)>) {
        for (id, username, devicename) in devices {
            if username == "" {
                // it's an anon user.
                self.anonymous_user.devices.push(Device::new(&id));
                info!("new anonymous user: {}", id);
            } else {
                let mut already_present = false;
                let mut device = Device::new(&id);
                device.name = devicename;
                // Add a device to a known User
                for registered in &mut self.registered_users {
                    info!("update account {} with device {} ({})", registered.name, device.name, device.ring_id);
                    if registered.name == username {
                        registered.devices.push(device.clone());
                        already_present = true;
                        break;
                    }
                }
                // User not found, create it
                if !already_present {
                    info!("create account {} with device {} ({})", username, device.name, device.ring_id);
                    let mut user = User {
                        name: username,
                        devices: Vec::new(),
                    };
                    user.devices.push(device);
                    self.registered_users.push(user);
                }
            }
        }
    }


// Private stuff

    /**
     * Add a new contact
     * @param self
     * @param account_id the account who accepts the request
     * @param from the contact to accept
     */
    fn add_contact(&self, account_id: &str, from: &str) {
        let method = "addContact";
        let dbus_msg = Message::new_method_call(self.ring_dbus, self.configuration_path, self.configuration_iface,
                                                method);
        if !dbus_msg.is_ok() {
            error!("method call fails. Please verify daemon's API.");
            return;
        }
        let conn = Connection::get_private(BusType::Session);
        if !conn.is_ok() {
            error!("connection not ok.");
            return;
        }
        let dbus = conn.unwrap();
        let _ = dbus.send_with_reply_and_block(
            dbus_msg.unwrap().append2(account_id, from), 2000).unwrap();
    }

    /**
     * Move the anonymous device to a registered user
     * @param self
     * @param ring_id to move (must be in anonymous)
     * @param username new device username
     */
    fn move_ring_to_user(&mut self, ring_id: &String, username: &String) {
        // Remove from anonymous_user
        let index = self.anonymous_user.devices.iter().position(|d| d.ring_id == *ring_id).unwrap();
        self.anonymous_user.devices.remove(index);
        // Update device for user
        for registered in &mut self.registered_users {
            if registered.name == *username {
                registered.devices.push(Device::new(ring_id));
            }
        }
        // Update database
        let _ = Database::update_username(ring_id, username);
    }

    /**
     * Handle link orders.
     * @param self
     * @param from_id the sender of the order
     * @param argument the ring_id if registered, the username if not
     */
    fn try_link_new_device(&mut self, from_id: &String, argument: &String) {
        // Retrieve users from database
        let account_id = self.account.id.clone();
        let (from_id, from_user, _) = Database::get_device(from_id);
        let mut do_push = true; // if we must store a temporary item to remember the order
        let mut do_clean = false; // user linked, the temporary item is not necessary anymore
        let linked_id : String;
        let linked_user : String;
        if from_user.len() == 0 {
            linked_id = from_id.clone();
            linked_user = argument.clone();
            // unknown want to be connected as user.
            for (id, account, authentified) in self.id_to_account_linker.clone() {
                if id == from_id && account == *argument {
                    do_push = false; // already here
                    if authentified {
                        // Do link and inform users
                        do_clean = true;
                        let msg = format!("{} linked to {}", from_id, argument);
                        info!("{}", msg);
                        self.move_ring_to_user(&id, &account);
                        self.send_interaction(&*account_id, &*id, &*msg);
                        break;
                    }
                }
            }
        } else {
            linked_id = argument.clone();
            linked_user = from_user.clone();
            // known user want a new device
            for (id, account, authentified) in self.id_to_account_linker.clone() {
                if id == *argument && account == from_user {
                    do_push = false; // already here
                    if !authentified {
                        // Do link
                        do_clean = true;
                        let msg = format!("{} linked to {}", argument, from_user);
                        info!("{}", msg);
                        self.move_ring_to_user(&id, &account);
                        self.send_interaction(&*account_id, &*id, &*msg);
                        break;
                    }
                }
            }
        }
        if do_push {
            // Remember this order and wait for the order of the other device
            info!("{} wants to be linked to {}", linked_id, linked_user);
            self.id_to_account_linker.push((linked_id.clone(), linked_user.clone(), from_user.len() != 0));
        }
        if do_clean {
            // Already linked, clean id_to_account_linker
            if let Some(index) = self.id_to_account_linker.iter()
                                .position(|i| i.0 == *linked_id && i.1 == *linked_user) {
                self.id_to_account_linker.remove(index);
            }
        }
    }

    /**
     * Try to give a new name to a device and inform the device
     * @param self
     * @param from_id the device which asks (must be registered)
     * @param ring_id to register
     * @param devicename new devicename
     */
    fn try_register_device(&mut self, from_id: &String, ring_id: &String, username: &String, devicename: &String) {
        let id = self.account.id.clone();
        let (from_id, from_user, _) = Database::get_device(from_id);
        let (mod_id, mod_user, _) = Database::get_device(ring_id);
        // Check if devices are for the same user
        if from_user != &*mod_user {
            let err = format!("!!!!!{} trying to register device with different user ({}) ", from_id, mod_id);
            warn!("{}", err);
            self.send_interaction(&*id, &*from_id, &*err);
            self.send_interaction(&*id, &*mod_id, &*err);
            return;
        }
        // Search if it's already registered
        if self.get_ring_id(&format!("{}_{}", username, devicename)).len() > 0 {
            let err = format!("registering {} for {} failed because devicename was found", devicename, ring_id);
            warn!("{}", err);
            self.send_interaction(&*id, ring_id, &*err);
        } else {
            // register device
            match Database::update_devicename(ring_id, devicename) {
                Ok(_) => {
                    // Update device for user
                    for registered in &mut self.registered_users {
                        if registered.name == *username {
                            for device in &mut registered.devices {
                                if device.ring_id == *ring_id {
                                    device.name = devicename.clone();
                                    break;
                                }
                            }
                        }
                    }
                    // And inform user
                    let msg = format!("{} is now known as {}_{}", ring_id, username, devicename);
                    info!("{}", msg);
                    self.send_interaction(&*id, &*from_id, &*msg);
                },
                _ => {
                    let err = format!("registering {} for {} failed when updating db", devicename, ring_id);
                    warn!("{}", err);
                    self.send_interaction(&*id, &*from_id, &*err);
                }
            }
        }
    }

    /**
     * Try to link to a new User a device and inform the device
     * @param self
     * @param ring_id to register
     * @param username new username
     */
    fn try_register_username(&mut self, ring_id: &String, username: &String) {
        let id = self.account.id.clone();
        // Lookup if it's already registered
        if self.get_ring_id(username).len() > 0 {
            let err = format!("registering {} for {} failed because username was found", username, ring_id);
            warn!("{}", err);
            self.send_interaction(&*id, ring_id, &*err);
        } else {
            // Register!
            match Database::update_username(ring_id, username) {
                Ok(_) => {
                    // Remove from anonymous_user
                    let index = self.anonymous_user.devices.iter().position(|d| d.ring_id == *ring_id).unwrap();
                    self.anonymous_user.devices.remove(index);
                    // Create new user
                    let mut new_user = User::new();
                    new_user.name = username.clone();
                    new_user.devices.push(Device::new(ring_id));
                    self.registered_users.push(new_user);
                    // Inform user that they is registered.
                    let msg = format!("{} is now known as {}", ring_id, username);
                    info!("{}", msg);
                    self.send_interaction(&*id, ring_id, &*msg);
                },
                _ => {
                    let err = format!("registering {} for {} failed when updating db", username, ring_id);
                    warn!("{}", err);
                    self.send_interaction(&*id, ring_id, &*err);
                }
            }
        }
    }

    /**
     * Try to remove a device for a user
     * @param self
     * @param from_id the device which asks (must be registered)
     * @param ring_id to revoke
     */
    fn try_remove_device(&mut self, from_id: &String, ring_id: &String) {
        let id = self.account.id.clone();
        let (from_id, from_user, _) = Database::get_device(from_id);
        let (mod_id, mod_user, _) = Database::get_device(ring_id);
        // Test if it's for a same user
        if from_user != &*mod_user {
            let err = format!("!!!!!{} trying to revoke device with different user ({}) ", from_id, mod_id);
            warn!("{}", err);
            self.send_interaction(&*id, &*from_id, &*err);
            self.send_interaction(&*id, &*mod_id, &*err);
            return;
        }
        // Remove the device
        let mut success = false;
        let mut remove_user = false;
        let mut user_idx = 0;
        for registered in &mut self.registered_users {
            let mut idx = 0;
            for device in &mut registered.devices {
                if device.ring_id == *ring_id {
                    device.name = String::new();
                    let _ = Database::update_devicename(ring_id, &String::new());
                    success = true;
                    break;
                }
                idx += 1;
            }
            if success {
                // Update devices
                registered.devices.remove(idx);
                if registered.devices.len() == 0 {
                    remove_user = true;
                }
                self.anonymous_user.devices.push(Device::new(ring_id));
                break;
            }
            user_idx += 1;
        }
        if remove_user {
            self.registered_users.remove(user_idx);
        }
        // And inform user
        let mut msg = format!("{} device revokation failed", ring_id);
        if success {
            msg = format!("{} device name revoked", ring_id);
            info!("{}", msg);
        } else {
            warn!("{}", msg);
        }
        self.send_interaction(&*id, ring_id, &*msg);
    }

    /**
     * Try to remove a user and its devices
     * @param self
     * @param ring_id to revoke
     */
    fn try_unregister(&mut self, ring_id: &String) {
        let id = self.account.id.clone();
        // Search username
        let mut name = String::new();
        for registered in &mut self.registered_users {
            for device in &mut registered.devices {
                if device.ring_id == *ring_id {
                    name = registered.name.clone();
                    break;
                }
            }
            if name.len() > 0 {
                break;
            }
        }
        // anonymous
        if name.len() == 0 {
            return;
        }

        let mut idx = 0;
        // Update registered_users and anonymous
        for registered in &mut self.registered_users.clone() {
            if registered.name == &*name {
                for device in &mut registered.devices {
                    let _ = Database::update_username(ring_id, &String::new());
                    let _ = Database::update_devicename(ring_id, &String::new());
                    self.anonymous_user.devices.push(Device::new(&ring_id));
                    info!("update device {} for {}", device.ring_id, registered.name);
                }
                let mut msg = format!("{} unregistered", registered.name);
                self.send_interaction(&*id, ring_id, &*msg);
                break;
            }
            idx += 1;
        }
        self.registered_users.remove(idx);
    }

    /**
     * Send a new text message
     * TODO handle multi types
     * @param self
     * @param from the account who send this
     * @param destination ring_id of the destination
     * @param body text to send
     * @return the interaction id if success. TODO, watch message status (if received)
     */
    fn send_interaction(&self, from: &str, destination: &str, body: &str) -> u64 {
        let mut payloads: HashMap<&str, &str> = HashMap::new();
        payloads.insert("text/plain", body);
        let payloads = Dict::new(payloads.iter());

        let dbus_msg = Message::new_method_call(self.ring_dbus, self.configuration_path, self.configuration_iface,
                                                "sendTextMessage");
        if !dbus_msg.is_ok() {
            error!("sendTextMessage fails. Please verify daemon's API.");
            return 0;
        }
        let conn = Connection::get_private(BusType::Session);
        if !conn.is_ok() {
            return 0;
        }
        let dbus = conn.unwrap();
        let response = dbus.send_with_reply_and_block(dbus_msg.unwrap().append3(from, destination, payloads), 2000).unwrap();
        // sendTextMessage returns one argument, which is a u64.
        let interaction_id: u64  = match response.get1() {
            Some(interaction_id) => interaction_id,
            None => 0
        };
        interaction_id
    }
}
