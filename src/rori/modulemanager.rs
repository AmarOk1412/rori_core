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
use rori::database::Database;
use rori::interaction::Interaction;
use rori::module::Module;
use std::thread;

/**
 * Class used to handle an interaction with the module activation loop
 */
pub struct ModuleManager {
    pub interaction: Interaction,
}

impl ModuleManager {
    /**
     * Generates a new ModuleManager
     * @return ModuleManager
     */
    pub fn new(interaction: Interaction) -> ModuleManager {
        ModuleManager {
            interaction: interaction,
        }
    }

    /**
     * Execute module activation loop
     * @param self
     */
    pub fn process(&self) {
        let mut priority = 0;
        let mut stop = false;
        // TODO, get priority list
        let max_priority = Database::get_max_priority() as u64;
        while !stop {
            // Get modules for this priority
            let modules = Database::get_enabled_modules(priority);
            if priority > max_priority {
                info!("No more modules to test");
                return;
            }
            // Test each modules
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
