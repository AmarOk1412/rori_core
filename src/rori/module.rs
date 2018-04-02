use cpython::{PyDict, Python};
use regex::Regex;
use rori::interaction::Interaction;
use serde_json;

pub trait Condition : Send + Sync {
    fn is_fulfilled_by(&self, interaction: &Interaction) -> bool;
}

pub struct TextCondition {
    condition: String,
}

impl Condition for TextCondition {
    fn is_fulfilled_by(&self, interaction: &Interaction) -> bool {
        let re = Regex::new(&*self.condition).unwrap();
        re.is_match(&*interaction.body.to_lowercase())
    }
}

impl TextCondition {
    pub fn new(condition: String) -> TextCondition {
        TextCondition {
            condition: condition,
        }
    }
}

pub struct Module {
    pub condition: Box<Condition>,
    pub name: String,
    pub path: String,
    pub priority: u64,
    pub enabled: bool,
}

impl Module {
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
        let load_module = py.import("rori_modules.load_module").unwrap();
        let interaction = serde_json::to_string(&interaction).unwrap_or(String::new());
        let continue_processing: bool =
            load_module.call(py, "exec_module", (self.path.clone(), interaction), None)
                .unwrap()
                .extract(py)
                .unwrap();
        continue_processing
    }
}
