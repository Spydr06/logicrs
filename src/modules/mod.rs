pub mod builtin;

use serde::{Serialize, Deserialize};

#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd)]
pub struct Module {
    name: String,
    builtin: bool,
    num_inputs: u8,
    num_outputs: u8
}

impl Module {
    pub fn new(name: String, num_inputs: u8, num_outputs: u8) -> Self {
        Self {
            name,
            builtin: false,
            num_inputs,
            num_outputs
        }
    }

    pub fn new_builtin(name: String, num_inputs: u8, num_outputs: u8) -> Self {
        Self {
            name,
            builtin: true,
            num_inputs,
            num_outputs
        }
    }

    pub fn builtin(&self) -> bool {
        self.builtin
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn get_num_inputs(&self) -> u8 {
        self.num_inputs
    }

    pub fn get_num_outputs(&self) -> u8 {
        self.num_outputs
    }
}

impl Ord for Module {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.chars().nth(0).unwrap().cmp(&other.name().chars().nth(0).unwrap())
    }
}

impl Eq for Module {}
