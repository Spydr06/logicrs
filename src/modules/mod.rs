pub mod builtin;

use crate::simulator::*;

use serde::{Serialize, Deserialize};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Module {
    name: String,
    builtin: bool,
    plot: Option<Plot>,
    num_inputs: u8,
    num_outputs: u8
}

impl Module {
    pub fn new(name: String, num_inputs: u8, num_outputs: u8) -> Self {
        Self {
            name,
            builtin: false,
            plot: Some(Plot::new()),
            num_inputs,
            num_outputs
        }
    }

    pub fn new_builtin<'a>(name: &'a str, num_inputs: u8, num_outputs: u8) -> Self {
        Self {
            name: name.to_string(),
            builtin: true,
            plot: None,
            num_inputs,
            num_outputs
        }
    }

    pub fn plot(&self) -> &Option<Plot> {
        &self.plot
    }

    pub fn plot_mut(&mut self) -> &mut Option<Plot> {
        &mut self.plot
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

impl PartialEq for Module {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl PartialOrd for Module {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.name.cmp(other.name()))
    }
}

impl Clone for Module {
    fn clone(&self) -> Self {
        if self.plot.is_some() {
            panic!("Tried to call `.clone()` on Module with Some(Plot)");
        }

        Self { 
            name: self.name.clone(),
            builtin: self.builtin.clone(),
            plot: None,
            num_inputs: self.num_inputs.clone(),
            num_outputs: self.num_outputs.clone()
        }
    }
}
