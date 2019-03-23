use core::rori::account::Account;
use dbus::arg::{Array, Dict};
use dbus::{Connection, BusType, NameFlag, tree,};
use dbus::tree::Factory;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

// Our storage device
#[derive(Debug)]
pub struct Storage {
    pub new_info: Arc<AtomicBool>,
    pub contacts_added: Vec<(String, String)>,
    pub interactions_sent: Vec<(String, String)>,
    pub account: Arc<Mutex<Account>>,
    pub accounts_added: Vec<HashMap<String, String>>,
    pub request_accepted: Vec<String>,
}

// Every storage device has its own object path.
// We therefore create a link from the object path to the Device.
#[derive(Copy, Clone, Default, Debug)]
struct TData;
impl tree::DataType for TData {
    type Tree = ();
    type ObjectPath = Arc<Mutex<Storage>>;
    type Property = ();
    type Interface = ();
    type Method = ();
    type Signal = ();
}

/**
 * Mock a ring daemon
 * tests will modify this structure
 */
pub struct Daemon {
    pub stop: Arc<AtomicBool>,
    pub initialized: Arc<AtomicBool>,
    pub storage: Arc<Mutex<Storage>>,
    emit_incoming_trust_request: Arc<AtomicBool>,
    emit_incoming_account_message: Vec<(String, String)>,
}

impl Daemon {
    /**
     * Init a daemon's mock
     */
    pub fn new() -> Daemon {
        let glados_account = Account {
            id: String::from("GLaDOs_id"),
            ring_id: String::from("GLaDOs_hash"),
            alias: String::from("GLaDOs"),
            enabled: false
        };
        Daemon {
            stop: Arc::new(AtomicBool::new(false)),
            initialized: Arc::new(AtomicBool::new(false)),
            storage: Arc::new(Mutex::new(Storage {
                new_info: Arc::new(AtomicBool::new(false)),
                contacts_added: Vec::new(),
                interactions_sent: Vec::new(),
                account: Arc::new(Mutex::new(glados_account)),
                accounts_added: Vec::new(),
                request_accepted: Vec::new(),
            })),
            emit_incoming_trust_request: Arc::new(AtomicBool::new(false)),
            emit_incoming_account_message: Vec::new(),
        }
    }

    /**
     * Run the mock
     * @param daemon the daemon to run
     */
    pub fn run(daemon: Arc<Mutex<Daemon>>) {
        let connection = Connection::get_private(BusType::Session).unwrap();
        let ring_dbus = "cx.ring.Ring";
        let configuration_path = "/cx/ring/Ring/ConfigurationManager";
        let configuration_iface = "cx.ring.Ring.ConfigurationManager";
        connection.register_name(ring_dbus, NameFlag::ReplaceExisting as u32).unwrap();
        let f = Factory::new_fn::<(TData)>();

        let incoming_trust_request = Some(Arc::new(
            f.signal("incomingTrustRequest", ())
             .arg(("accountID", "s"))
             .arg(("from", "s"))
             .arg(("payload", "ay"))
             .arg(("receiveTime", "t"))
        ));
        let signal_incoming_trust_request = incoming_trust_request.clone().unwrap();

        let incoming_account_message = Some(Arc::new(
            f.signal("incomingAccountMessage", ())
             .arg(("accountID", "s"))
             .arg(("from", "s"))
             .arg(("payload", "ay"))
        ));
        let signal_incoming_account_message = incoming_account_message.clone().unwrap();
        let storage = daemon.lock().unwrap().storage.clone();

        let add_contact = f.method("addContact", (), move |m| {
                                let storage: &Arc<Mutex<Storage>> = m.path.get_data();
                                let (account_id, id) = m.msg.get2::<&str, &str>();
                                storage.lock().unwrap().contacts_added.push((String::from(account_id.unwrap()), String::from(id.unwrap())));
                                storage.lock().unwrap().new_info.store(true, Ordering::SeqCst);
                                let rm = m.msg.method_return();
                                Ok(vec!(rm))
                            })
                           .in_arg(("accountID", "s"))
                           .in_arg(("uri", "s"));

       let add_account = f.method("addAccount", (), move |m| {
                               let storage: &Arc<Mutex<Storage>> = m.path.get_data();
                               let details = m.msg.get1::<Dict<&str, &str, _>>();
                               let mut account_added: HashMap<String, String> = HashMap::new();
                               for detail in details.unwrap() {
                                   match detail {
                                       (key, value) => {
                                           account_added.insert(String::from(key), String::from(value));
                                       }
                                   }
                               }
                               storage.lock().unwrap().accounts_added.push(account_added);
                               storage.lock().unwrap().new_info.store(true, Ordering::SeqCst);
                               let rm = m.msg.method_return();
                               let rm = rm.append1(storage.lock().unwrap().accounts_added.len().to_string());
                               Ok(vec!(rm))
                           })
                          .in_arg(("details", "a{ss}"))
                          .out_arg(("id", "s"));

       let send_text_message = f.method("sendTextMessage", (), move |m| {
                               let storage: &Arc<Mutex<Storage>> = m.path.get_data();
                               let (from, to, _) = m.msg.get3::<&str, &str, Dict<&str, &str, _>>();
                               storage.lock().unwrap().interactions_sent.push((String::from(from.unwrap()), String::from(to.unwrap())));
                               storage.lock().unwrap().new_info.store(true, Ordering::SeqCst);
                               let rm = m.msg.method_return();
                               Ok(vec!(rm))
                           })
                          .in_arg(("accountID", "s"))
                          .in_arg(("uri", "s"));

        let get_account_list = f.method("getAccountList", (), move |m| {
                                     let storage: &Arc<Mutex<Storage>> = m.path.get_data();
                                     let account = storage.lock().unwrap().account.clone();
                                     let account = account.lock().unwrap();
                                     let mut accounts: Vec<&str> = Vec::new();
                                     accounts.push(&*account.id);
                                     let details = Array::new(accounts.iter());
                                     let rm = m.msg.method_return();
                                     let rm = rm.append1(details);
                                     Ok(vec!(rm))
                                 })
                                 .out_arg(("account_list", "as"));

        let get_account_details = f.method("getAccountDetails", (), move |m| {
                                       let storage: &Arc<Mutex<Storage>> = m.path.get_data();
                                       let account_id = m.msg.get1::<&str>();
                                       let account = storage.lock().unwrap().account.clone();
                                       let account = account.lock().unwrap();
                                       let mut details: HashMap<&str, &str> = HashMap::new();
                                       if account.id == account_id.unwrap_or("") {
                                           if account.enabled {
                                               details.insert("Account.enable", "true");
                                           } else {
                                               details.insert("Account.enable", "false");
                                           }
                                           details.insert("Account.alias", &*account.alias);
                                           details.insert("Account.username", &*account.ring_id);
                                       }
                                       let details = Dict::new(details.iter());
                                       let rm = m.msg.method_return();
                                       let rm = rm.append1(details);
                                       Ok(vec!(rm))
                                   })
                                   .in_arg(("accountID", "s"))
                                   .out_arg(("details", "a{ss}"));

        let get_contacts = f.method("getContacts", (), move |m| {
                                        let mut details = Vec::new();
                                        let mut contact = HashMap::new();
                                        contact.insert("id", "Atlas");
                                        details.push(contact);
                                        let mut contact = HashMap::new();
                                        contact.insert("id", "PBody");
                                        details.push(contact);
                                        let mut contact = HashMap::new();
                                        contact.insert("id", "Weasley");
                                        details.push(contact);
                                        let mut contact = HashMap::new();
                                        contact.insert("id", "Space core");
                                        details.push(contact);
                                        let rm = m.msg.method_return();
                                        let rm = rm.append1(details);
                                        Ok(vec!(rm))
                                    })
                                    .in_arg(("accountID", "s"))
                                    .out_arg(("details", "aa{ss}"));

        let send_register = f.method("sendRegister", (), move |m| {
                                  let storage: &Arc<Mutex<Storage>> = m.path.get_data();
                                  let (account_id, enabled) = m.msg.get2::<&str, bool>();
                                  let account = &storage.lock().unwrap().account;
                                  let id = account.lock().unwrap().id.clone();
                                  if id == account_id.unwrap_or("") {
                                      account.lock().unwrap().enabled = enabled.unwrap_or(false);
                                  }
                                  let rm = m.msg.method_return();
                                  Ok(vec!(rm))
                              })
                              .in_arg(("accountID", "s"))
                              .in_arg(("enable", "b"));

        let accept_trust_request = f.method("acceptTrustRequest", (), move |m| {
                                   let storage: &Arc<Mutex<Storage>> = m.path.get_data();
                                   let (_, from) = m.msg.get2::<&str, &str>();
                                   let rm = m.msg.method_return();
                                   let rm = rm.append1(true);
                                   storage.lock().unwrap().request_accepted.push(from.unwrap_or("").to_string());
                                   Ok(vec!(rm))
                               })
                               .in_arg(("accountID", "s"))
                               .in_arg(("from", "s"))
                               .out_arg(("success", "v"));

        // We create a tree with one object path inside and make that path introspectable.
        let tree = f.tree(())
                    .add(f.object_path(configuration_path, storage.clone()).introspectable().add(
                        // We add an interface to the object path...
                        f.interface("cx.ring.Ring.ConfigurationManager", ())
                         .add_m(send_text_message)
                         .add_m(add_account)
                         .add_m(add_contact)
                         .add_m(get_account_list)
                         .add_m(get_account_details)
                         .add_m(get_contacts)
                         .add_m(send_register)
                         .add_m(accept_trust_request)
                         .add_s(signal_incoming_trust_request)
                         .add_s(signal_incoming_account_message)
                    ));

        // We register all object paths in the tree.
        tree.set_registered(&connection, true).unwrap();

        // We add the tree to the connection so that incoming method calls will be handled
        // automatically during calls to "incoming".
        connection.add_handler(tree);
        daemon.lock().unwrap().initialized.store(true, Ordering::SeqCst);

        // Serve other peers forever.
        loop {
            connection.incoming(100).next();
            let emit_incoming_trust_request = daemon.lock().unwrap().emit_incoming_trust_request.load(Ordering::SeqCst);
            let emit_incoming_account_message = daemon.lock().unwrap().emit_incoming_account_message.clone();
            if emit_incoming_trust_request {
                let storage = daemon.lock().unwrap().storage.clone();
                storage.lock().unwrap().request_accepted = Vec::new();
                let signal = incoming_trust_request.clone().unwrap();
                let path = configuration_path.to_string().into();
                let iface = configuration_iface.to_string().into();
                let dict = Dict::new(vec![("", "")]);
                let msg = signal.msg(&path, &iface).append2("GLaDOs_id", "Eve").append2(dict, 0);
                let _ = connection.send(msg).map_err(|_| "Sending DBus signal failed");
                daemon.lock().unwrap().emit_incoming_trust_request.store(false, Ordering::SeqCst);
            }
            if emit_incoming_account_message.len() > 0 {
                let signal = incoming_account_message.clone().unwrap();
                let path = configuration_path.to_string().into();
                let iface = configuration_iface.to_string().into();
                let content = emit_incoming_account_message.first().unwrap();
                let (datatype, body) = (content.0.clone(), content.1.clone());
                let dict = Dict::new(vec![(&*datatype, &*body)]);
                let msg = signal.msg(&path, &iface).append3("GLaDOs_id", "Eve", dict);
                let _ = connection.send(msg).map_err(|_| "Sending DBus signal failed");
                daemon.lock().unwrap().emit_incoming_account_message = Vec::new();
            }

            let stop = daemon.lock().unwrap().stop.load(Ordering::SeqCst);
            if stop {
                return;
            }
        }
    }

    /**
     * emit incomingTrustRequest()
     * @param self
     */
    #[allow(dead_code)]
    pub fn emit_incoming_trust_request(&mut self) {
        self.emit_incoming_trust_request.store(true, Ordering::SeqCst);
    }

    /**
     * emit incomingAccountMessage()
     * @param self
     * @param datatype
     * @param body
     */
    #[allow(dead_code)]
    pub fn emit_incoming_account_message(&mut self, datatype: &String, body: &String) {
        self.emit_incoming_account_message.push((datatype.clone(), body.clone()));
    }

    /**
     * Stop the execution of the mock
     * @param self
     */
    pub fn stop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
    }
}
