use crate::simulator::{*, builtin::BUILTINS};

use serde::{Serialize, Deserialize};

pub type SimulatorFn = fn(u128, &mut Block) -> u128;

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Module {
    name: String,
    builtin: bool,
    hidden: bool,
    plot: Option<Plot>,
    num_inputs: u8,
    num_outputs: u8,
    decoration: Decoration,
}

impl Module {
    pub fn new(name: String, num_inputs: u8, num_outputs: u8) -> Self {
        Self {
            name,
            builtin: false,
            hidden: false,
            plot: Some(Plot::new()),
            num_inputs,
            num_outputs,
            decoration: Decoration::None
        }
    }

    pub fn new_builtin<'a>(name: &'a str, hidden: bool, num_inputs: u8, num_outputs: u8, decoration: Decoration) -> Self {
        Self {
            name: name.to_string(),
            builtin: true,
            hidden,
            plot: None,
            num_inputs,
            num_outputs,
            decoration
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

    pub fn hidden(&self) -> bool {
        self.hidden
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

    pub fn decoration(&self) -> &Decoration {
        &self.decoration
    }

    pub fn simulate(&self, inputs: u128, instance: &mut Block) -> u128 {
        let outputs = 
        if self.builtin && let Some(builtin) = BUILTINS.get(self.name.as_str()) {
            builtin.simulate(inputs, instance)
        }
        else {
            error!("custom modules are not supported currently");
            0
        };

        info!("simulate module {} with inputs: {inputs:#b} generates: {outputs:#b}", self.name);
        outputs
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