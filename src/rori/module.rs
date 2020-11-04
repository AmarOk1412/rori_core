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
use cpython::{PyDict, Python};
use regex::Regex;
use rori::interaction::Interaction;
use serde_json;

/**
 * Condition's trait to implement
 */
pub trait Condition : Send + Sync {
    fn is_fulfilled_by(&self, interaction: &Interaction) -> bool;
}

/**
 * Condition for text modules
 */
pub struct TextCondition {
    condition: String,
}

/**
 * This condition is fulfilled if the interaction's body matchs the regex of the module
 */
impl Condition for TextCondition {
    fn is_fulfilled_by(&self, interaction: &Interaction) -> bool {
        let re = Regex::new(&*self.condition).unwrap();
        re.is_match(&*interaction.body.to_lowercase())
    }
}

impl TextCondition {
    /**
     * Return a new TextCondition
     * @param condition
     * @return TextCondition
     */
    pub fn new(condition: String) -> TextCondition {
        TextCondition {
            condition: condition,
        }
    }
}

/**
 * Represents a Module
 */
pub struct Module {
    pub condition: Box<dyn Condition>,
    pub name: String,
    pub path: String,
    pub priority: u64,
    pub enabled: bool,
}

impl Module {
    /**
     * Execute the module and get if we should continue to process other modules
     * @param self
     * @param interaction which has trigerred this module
     * @return if we continue to process the interaction (true on error to avoid to stop other modules)
     */
    pub fn exec(&self, interaction: &Interaction) -> bool {
        // Init python module
        let py = Python::acquire_gil();
        let py = py.python();
        // Add . to PYTHONPATH to be able to load modules
        let _ = py.import("sys").unwrap();
        let locals = PyDict::new(py);
        locals.set_item(py, "sys", py.import("sys").unwrap()).unwrap();
        py.eval("sys.path.append('.')", None, Some(&locals)).unwrap();
        py.eval("sys.path.append('./rori_modules/')", None, Some(&locals)).unwrap();
        // This will execute the linked module
        let load_module = py.import("rori_modules.load_module");
        if !load_module.is_ok() {
            error!("Error loading module {}", self.name);
            return true;
        }
        let load_module = load_module.unwrap();
        let interaction = serde_json::to_string(&interaction).unwrap_or(String::new());
        let continue_processing = load_module.call(py, "exec_module", (self.path.clone(), interaction), None);
        if !continue_processing.is_ok() {
            error!("Error while executing module {}", self.name);
            return true;
        }
        let continue_processing = continue_processing.unwrap().extract(py);
        if !continue_processing.is_ok() {
            error!("Error while getting result for module {}", self.name);
            return true;
        }
        continue_processing.unwrap()
    }
}
