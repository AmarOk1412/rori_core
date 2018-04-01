use rori::database::Database;
use rori::interaction::Interaction;
use rori::module::Module;
use std::thread;

pub struct ModuleManager {
    pub interaction: Interaction,
}

impl ModuleManager {
    pub fn new(interaction: Interaction) -> ModuleManager {
        ModuleManager {
            interaction: interaction,
        }
    }

    pub fn process(&self) {
        let mut priority = 0;
        let mut stop = false;
        let max_priority = Database::get_max_priority() as u64;
        while !stop {
            let modules = Database::get_enabled_modules(priority);
            if priority > max_priority {
                info!("No more modules to test");
                return;
            }
            let mut children = vec![];
            for module in modules {
                let interaction = self.interaction.clone();
                children.push(thread::spawn(move || {
                    if module.condition.is_fulfilled_by(&interaction) {
                        info!("{} module's condition fulfilled. Exec module", module.name);
                        let result = module.exec(&interaction);
                        if !result {
                            info!("{} asks RORI to stop. Stopping at the next priority...", module.name);
                            stop = true;
                        }
                    } else {
                        info!("{} module's condition not fulfilled.", module.name);
                    }
                }));
            }
            for child in children {
                // Wait for the thread to finish. Returns a result.
                let _ = child.join();
            }
            priority += 1;
        }
        info!("Stopping processing")
    }
}
