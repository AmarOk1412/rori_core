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
use rori::account::Account;
use rori::database::Database;
use rori::interaction::Interaction;
use rori::modulemanager::ModuleManager;
use rori::user::{Device, User};
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
    id_to_account_linker: Vec<(String, String, bool, String)>
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
        let insert_into_db = Database::insert_new_device(ring_id, &String::new(), &String::new(), false);
        match insert_into_db {
            Ok(i) => {
                self.anonymous_user.devices.push(Device::new(&i, &ring_id));
            }
            _ => {
                error!("add_new_anonymous_device failed");
                return false;
            }
        }
        info!("{} added to anonymouses", ring_id);
        true
    }

    /**
     * Retrieve a ring_id for a given username or devicename
     * @param self
     * @param name username or devicename to find
     * @return the ring_id if found, else an empty String
     */
    pub fn get_hash(&mut self, name: &String) -> String {
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
        let hash = interaction.device_author.ring_id.clone();
        let is_bridge = Database::is_bridge(&hash);
        let mut sub_author = String::new();

        if !is_bridge {
            let mut user_found = false;
            for device in &self.anonymous_user.devices {
                if &*hash == &*device.ring_id {
                    user_found = true;
                    break;
                }
            }
            if !user_found {
                // User not found, continue to search.
                for mut registered in &self.registered_users {
                    for device in &registered.devices {
                        if &*hash == &*device.ring_id {
                            user_found = true;
                            username = registered.name.clone();
                            break;
                        }
                    }
                }
            }
            if !user_found {
                // User not found add it
                self.add_contact(&*self.account.id, &*hash);
                self.add_new_anonymous_device(&hash);
            }
        } else {
            sub_author = match interaction.metadatas.get("sa") {
                Some(sa) => sa.to_string(),
                None => String::new()
            };
            if sub_author.len() > 0 {
                username = Database::sub_author(&hash, &sub_author);
            }

            if !Database::is_bridge_with_username(&hash, &username) {
                warn!("{} is trying to talk for another user", hash);
                self.send_interaction(&*self.account.id, &hash,
                    &*format!("{{\"registered\":false, \"username\":\"{}\", \"err\":\"{} is not yours\"}}", username, username), "rori/message");
                return;
            }
        }

        let tuple = Database::get_device(&hash, &username);
        let mut new_interaction = interaction.clone();
        new_interaction.device_author = Device {
            id: tuple.0,
            name: tuple.3,
            ring_id: tuple.1,
            is_bridge: tuple.4 == 1
        };

        // TODO should be handle by a module
        if interaction.datatype == "rori/command" {
            if username.len() == 0 {
                // Anonymous to user
                if interaction.body.starts_with("/register") {
                    let split: Vec<&str> = interaction.body.split(' ').collect();
                    if split.len() < 2 {
                        warn!("register received, but no username detected");
                        return;
                    }
                    self.try_register_username(&hash,
                                               &String::from(*split.get(1).unwrap()), &sub_author);
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
                   let mut device_to_add = hash.clone();
                   if split.len() == 3 && split.last().unwrap_or(&"").len() > 0 {
                       device_to_add = split.last().unwrap_or(&"").to_string();
                   }
                   self.try_register_device(&hash, &device_to_add, &username,
                                            &String::from(*split.get(1).unwrap()));
                } else if interaction.body.starts_with("/rm_device") {
                    // User wants to revoke a device
                    // /rm_device name (ring_id)
                    let mut device_to_remove = hash.clone();
                    let split: Vec<&str> = interaction.body.split(' ').collect();
                    if split.len() == 2 && split.last().unwrap_or(&"").len() > 0 {
                        device_to_remove = split.last().unwrap_or(&"").to_string();
                    }
                    self.try_remove_device(&hash, &device_to_remove, &username);
                } else if interaction.body.starts_with("/unregister") {
                    // User wants to unregister
                    self.try_unregister(&hash, &username);
                }
            }

            let device = Database::get_device(&hash, &username);

            if interaction.body.starts_with("/add_types") {
                // Handle add_type
                let mut split: Vec<&str> = interaction.body.split(' ').collect();
                split.remove(0);
                self.add_datatypes(&device.0, split);
            } else if interaction.body.starts_with("/bridgify") {
                // Handle add_type
                self.bridgify(&device.0);
            } else if interaction.body.starts_with("/rm_types") {
                // Handle rm_type
                let mut split: Vec<&str> = interaction.body.split(' ').collect();
                split.remove(0);
                self.rm_datatypes(&device.0, split);
            } else if interaction.body.starts_with("/set_types") {
                // Handle set_type
                let mut split: Vec<&str> = interaction.body.split(' ').collect();
                split.remove(0);
                self.set_datatypes(&device.0, split);
            } else if interaction.body.starts_with("/link") {
                // Handle multi-devices
                let split: Vec<&str> = interaction.body.split(' ').collect();
                if split.len() < 2 {
                    warn!("link received, but no argument detected");
                    return;
                }
                self.try_link_new_device(&hash,
                                         &String::from(*split.get(1).unwrap()), &username, &sub_author);
            }

        }

        let mm = ModuleManager::new(new_interaction);
        mm.process();
    }

    /**
     * Build users from given devices
     * NOTE: should be in database.
     * @param self
     * @param devices to process
     */
    pub fn load_devices(&mut self, devices: Vec<(i32, String, String, String, bool)>) {
        for (id, hash, username, devicename, _is_bridge) in devices {
            if username == "" {
                // it's an anon user.
                self.anonymous_user.devices.push(Device::new(&id, &hash));
                info!("new anonymous user: {}", hash);
            } else {
                let mut already_present = false;
                let mut device = Device::new(&id, &hash);
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
                                                method).ok().expect("method call fails. Please verify daemon's API.");
        let dbus = Connection::get_private(BusType::Session).ok().expect("connection not ok.");
        let _ = dbus.send_with_reply_and_block(
                    dbus_msg.append2(account_id, from), 2000
                ).unwrap();
    }

    /**
     * Add some datatypes of a device
     * @param device_id
     * @param add_types to add
     */
    fn add_datatypes(&self, device_id: &i32, add_types: Vec<&str>) {
        let mut current_datatypes = Database::get_datatypes(&device_id);
        for dtype in add_types.into_iter() {
            match current_datatypes.iter().position(|dt| dt == dtype) {
                Some(_) => {},
                None => current_datatypes.push(String::from(dtype))
            }
        }
        let _ = Database::set_datatypes(&device_id, current_datatypes);
    }

    /**
     * Change a device to a bridge
     * @param device_id
     */
    fn bridgify(&mut self, device_id: &i32) {
        // Update database
        let _ = Database::bridgify(&device_id);
        // Update anonymous user
        let index = self.anonymous_user.devices.iter().position(|d| d.id == *device_id).unwrap();
        let mut new_device = self.anonymous_user.devices.get(index).unwrap().clone();
        new_device.is_bridge = true;
        self.anonymous_user.devices.remove(index);
        self.anonymous_user.devices.push(new_device);
        info!("Device {} is now a bridge", &device_id);
    }

    /**
     * Move the anonymous device to a registered user
     * @param self
     * @param hash to move (must be in anonymous)
     * @param username new device username
     */
    fn move_ring_to_user(&mut self, hash: &String, username: &String) {
        let is_bridge = Database::is_bridge(hash);
        let did : i32;
        if is_bridge {
            let insert_into_db = Database::insert_new_device(hash, username, &String::new(), true);
            match insert_into_db {
                Ok(i) => {
                    did = i;
                }
                _ => {
                    error!("move_ring_to_user failed");
                    return;
                }
            }
        } else {
            // Remove from anonymous_user
            let index = self.anonymous_user.devices.iter().position(|d| d.ring_id == *hash).unwrap();
            did = self.anonymous_user.devices.get(index).unwrap().id;
            self.anonymous_user.devices.remove(index);
        }
        // Update device for user
        for registered in &mut self.registered_users {
            if registered.name == *username {
                registered.devices.push(Device::new(&did, hash));
            }
        }
        // Update database
        let _ = Database::update_username(&did, username);
    }

    /**
     * Remove some datatypes of a device
     * @param id
     * @param add_types to remove
     */
    fn rm_datatypes(&self, id: &i32, add_types: Vec<&str>) {
        let mut current_datatypes = Database::get_datatypes(id);
        for dtype in add_types.into_iter() {
            match current_datatypes.iter().position(|dt| dt == dtype) {
                Some(p) => {
                    current_datatypes.remove(p);
                },
                None => {}
            }
        }
        let _ = Database::set_datatypes(id, current_datatypes);
    }

    /**
     * Handle link orders.
     * @param self
     * @param from_id the sender of the order
     * @param argument the ring_id if registered, the username if not
     * @param username the username if bridge
     * @param sub_author the sub_author if bridge
     */
    fn try_link_new_device(&mut self, from_id: &String, argument: &String, username: &String, sub_author: &String) {
        // Retrieve users from database
        let id = self.account.id.clone();
        let is_bridge = Database::is_bridge(&from_id);
        let hash : String;
        let from_user : String;
        if is_bridge {
            let tuple = Database::get_device(&from_id, username);
            hash = tuple.1;
            from_user = tuple.2;
        } else {
            let tuple = Database::get_devices_for_hash(&from_id).get(0).unwrap().clone();
            hash = tuple.1;
            from_user = tuple.2;
        }

        // TODO inform user that a new device is linked
        let mut do_push = true; // if we must store a temporary item to remember the order
        let mut do_clean = false; // user linked, the temporary item is not necessary anymore
        let linked_id : String;
        let linked_user : String;
        if from_user.len() == 0 {
            linked_id = hash;
            linked_user = argument.clone();

            // unknown want to be connected as user.
            for (hash, account, authentified, sub_author) in self.id_to_account_linker.clone() {
                if hash == *from_id && account == *argument {
                    do_push = false; // already here
                    if authentified {
                        // Do link and inform users
                        do_clean = true;
                        let msg = format!("{} linked to {}", from_id, argument);
                        info!("{}", msg);
                        self.move_ring_to_user(&hash, &account);
                        let _ = Database::update_sub_author(&Database::get_device(&hash, argument).0, &sub_author);
                        self.send_interaction(&*id, &*hash, &*format!("{{\"registered\":true, \"username\":\"{}\", \"sa\":\"{}\"}}", argument, sub_author), "rori/message");
                        break;
                    }
                }
            }
        } else {
            linked_id = argument.clone();
            linked_user = from_user.clone();
            // known user want a new device
            for (hash, account, authentified, sub_author) in self.id_to_account_linker.clone() {
                if hash == *argument && account == from_user {
                    do_push = false; // already here
                    if !authentified {
                        // Do link
                        do_clean = true;
                        let msg = format!("{} linked to {}", argument, from_user);
                        info!("{}", msg);
                        self.move_ring_to_user(&hash, &account);
                        let _ = Database::update_sub_author(&Database::get_device(&hash, &from_user).0, &sub_author);
                        self.send_interaction(&*id, &*hash, &*format!("{{\"registered\":true, \"username\":\"{}\", \"sa\":\"{}\"}}", from_user, sub_author), "rori/message");
                        break;
                    }
                }
            }
        }
        if do_push {
            // Remember this order and wait for the order of the other device
            info!("{} wants to be linked to {}", linked_id, linked_user);
            self.id_to_account_linker.push((linked_id.clone(), linked_user.clone(), from_user.len() != 0, sub_author.clone()));
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
     * @param sub_author if bridge
     */
    fn try_register_device(&mut self, from_id: &String, ring_id: &String, username: &String, devicename: &String) {
        let id = self.account.id.clone();
        let (_, from_id, from_user, _, _) = Database::get_device(from_id, username);
        let (_, mod_id, mod_user, _, _) = Database::get_device(ring_id, username);
        // Check if devices are for the same user
        if from_user != &*mod_user {
            let err = format!("!!!!!{} trying to register device with different user ({}) ", from_id, mod_id);
            warn!("{}", err);
            self.send_interaction(&*id, &*from_id, &*format!("{{\"dregistered\":false, \"username\":\"{}\", \"err\":\"bad_register error from device {}\"}}", username, from_id), "rori/message");
            self.send_interaction(&*id, &*mod_id, &*format!("{{\"dregistered\":false, \"username\":\"{}\", \"err\":\"bad_register error from device {}\"}}", username, from_id), "rori/message");
            return;
        }
        // Search if it's already registered
        if self.get_hash(&format!("{}_{}", username, devicename)).len() > 0 {
            let err = format!("registering {} for {} failed because devicename was found", devicename, ring_id);
            warn!("{}", err);
            self.send_interaction(&*id, ring_id, &*format!("{{\"dregistered\":false, \"devicename\":\"{}\", \"err\":\"{}_{} already registered\"}}", devicename, username, devicename), "rori/message");
        } else {
            // search device to modify
            let device = Database::get_device(ring_id, username);
            // register device
            Database::update_devicename(&device.0, devicename)
            .ok().expect(&*format!("registering {}_{} for device {} failed when updating db", username, devicename, device.0));
            // Update device for user
            for registered in &mut self.registered_users {
                if registered.name == *username {
                    for d in &mut registered.devices {
                        if device.0 == d.id {
                            d.name = devicename.clone();
                            break;
                        }
                    }
                }
            }
            // And inform user
            let msg = format!("Device {} is now known as {}_{}", device.0.to_string(), username, devicename);
            info!("{}", msg);
            self.send_interaction(&*id, &*from_id, &*format!("{{\"dregistered\":true, \"devicename\":\"{}\"}}", devicename), "rori/message");
        }
    }

    /**
     * Try to link to a new User a device and inform the device
     * @param self
     * @param hash to register
     * @param username new username
     * @param sub_author if bridge
     */
    fn try_register_username(&mut self, hash: &String, username: &String, sub_author: &String) {

        let id = self.account.id.clone();
        let already_taken = self.get_hash(username).len() > 0;
        if already_taken {
            let err = format!("registering {} for {} failed because username was found", username, hash);
            warn!("{}", err);
            self.send_interaction(&*id, hash, &*format!("{{\"registered\":false, \"username\":\"{}\", \"err\":\"{} already registered\"}}", username, username), "rori/message");
        } else {
            // Register!
            let is_bridge = Database::is_bridge(hash);
            if is_bridge {
                // Add a new device for user
                // NOTE: do not remove from anonymouses, it's a bridge!
                let insert_into_db = Database::insert_new_device(hash, username, &String::new(), true);
                match insert_into_db {
                    Ok(i) => {
                        let mut new_user = User::new();
                        new_user.name = username.clone();
                        new_user.devices.push(Device::new(&i, hash));
                        self.registered_users.push(new_user);
                        let _ = Database::update_sub_author(&i, sub_author);
                    }
                    _ => {
                        error!("try_register_username failed when inserting new device");
                        return;
                    }
                }
            } else {
                // Remove from anonymous_user
                let index = self.anonymous_user.devices.iter().position(|d| d.ring_id == *hash).unwrap();
                let id = self.anonymous_user.devices.get(index).unwrap().id;
                self.anonymous_user.devices.remove(index);
                // Update database
                Database::update_username(&id, username)
                .ok().expect(&*format!("registering {} for {} failed when updating db", username, hash));
                // Create new user
                let mut new_user = User::new();
                new_user.name = username.clone();
                new_user.devices.push(Device::new(&id, hash));
                self.registered_users.push(new_user);
            }
            // Inform user that they is registered.
            let msg = format!("{} is now known as {}", hash, username);
            info!("{}", msg);
            self.send_interaction(&*id, hash, &*format!("{{\"registered\":true, \"username\":\"{}\", \"sa\":\"{}\"}}", username, sub_author), "rori/message");
        }
    }

    /**
     * Try to remove a device for a user
     * @param self
     * @param from_id the device which asks (must be registered)
     * @param ring_id to revoke
     * @param username asking
     */
    fn try_remove_device(&mut self, from_id: &String, ring_id: &String, username: &String) {
        let id = self.account.id.clone();
        let (_, from_id, from_user, _, _) = Database::get_device(from_id, username);
        let (mod_device_id, mod_id, mod_user, _, is_bridge) = Database::get_device(ring_id, username);
        let sub_author = Database::sub_author_id(&mod_id, &username);
        // Test if it's for a same user
        if from_user != &*mod_user {
            let err = format!("!!!!!{} trying to revoke device with different user ({}) ", from_id, mod_id);
            warn!("{}", err);
            return;
        }
        // Remove the device
        let mut success = false;
        let mut remove_user = false;
        let mut user_idx = 0;
        for registered in &mut self.registered_users {
            let mut idx = 0;
            for device in &mut registered.devices {
                if device.id == mod_device_id {
                    device.name = String::new();
                    if is_bridge == 1 {
                        let _ = Database::remove_device(&mod_device_id);
                    } else {
                        let _ = Database::update_devicename(&mod_device_id, &String::new());
                        let _ = Database::update_username(&mod_device_id, &String::new());
                    }
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
                if is_bridge != 1 {
                    self.anonymous_user.devices.push(Device::new(&mod_device_id, ring_id));
                }
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
            self.send_interaction(&*id, &*ring_id, &*format!("{{\"registered\":false, \"username\":\"{}\", \"sa\":\"{}\"}}", username, sub_author), "rori/message");
        } else {
            warn!("{}", msg);
        }
    }

    /**
     * Try to remove a user and its devices
     * @param self
     * @param hash of the device to revoke
     * @param username to revoke
     */
    fn try_unregister(&mut self, hash: &String, username: &String) {
        let mut name = String::new();
        let is_bridge = Database::is_bridge(hash);
        let sub_author = Database::sub_author_id(&hash, &username);
        if is_bridge {
            name = username.clone();
        } else {
            // Search username
            for registered in &mut self.registered_users {
                for device in &mut registered.devices {
                    if device.ring_id == *hash {
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
        }
        let id = self.account.id.clone();

        let mut idx = 0;
        // Update registered_users and anonymous
        for registered in &mut self.registered_users.clone() {
            if registered.name == &*name {
                for device in &mut registered.devices {
                    let is_bridge = Database::is_bridge(&device.ring_id);
                    if is_bridge {
                        let _ = Database::remove_device(&device.id);
                    } else {
                        let _ = Database::update_username(&device.id, &String::new());
                        let _ = Database::update_devicename(&device.id, &String::new());
                        self.anonymous_user.devices.push(Device::new(&device.id, &device.ring_id));
                    }
                    info!("update device {} for {}", device.ring_id, registered.name);
                }
                let mut msg = format!("{} unregistered", registered.name);
                info!("{}", msg);
                self.send_interaction(&*id, hash, &*format!("{{\"registered\":false, \"username\":\"{}\"\"sa\":\"{}\"}}", registered.name, sub_author), "rori/message");
                break;
            }
            idx += 1;
        }
        self.registered_users.remove(idx);
    }

    /**
     * Send a new text message
     * @param self
     * @param from the account who send this
     * @param destination ring_id of the destination
     * @param body text to send
     * @param datatype of the message
     * @return the interaction id if success. TODO, watch message status (if received)
     */
    fn send_interaction(&self, from: &str, destination: &str, body: &str, datatype: &str) -> u64 {
        let mut payloads: HashMap<&str, &str> = HashMap::new();
        payloads.insert(datatype, body);
        let payloads = Dict::new(payloads.iter());

        let dbus_msg = Message::new_method_call(self.ring_dbus, self.configuration_path, self.configuration_iface,
                                                "sendTextMessage").ok().expect("sendTextMessage fails. Please verify daemon's API.");
        let dbus = Connection::get_private(BusType::Session).ok().expect("connection failed");
        let response = dbus.send_with_reply_and_block(dbus_msg.append3(from, destination, payloads), 2000).unwrap();
        // sendTextMessage returns one argument, which is a u64.
        let interaction_id: u64  = match response.get1() {
            Some(interaction_id) => interaction_id,
            None => 0
        };
        interaction_id
    }

    /**
     * Change the datatypes of a device
     * @param id
     * @param datatypes
     */
    fn set_datatypes(&self, id: &i32, datatypes: Vec<&str>) {
        let mut dt: Vec<String> = Vec::new();
        for datatype in datatypes.into_iter() {
            dt.push(String::from(datatype));
        }
        let _ = Database::set_datatypes(&id, dt);
    }
}
