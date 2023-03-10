use std::collections::HashSet;

use crate::simulator::{*, builtin::BUILTINS};

use serde::{Serialize, Deserialize};

pub type SimulatorFn = fn(u128, &mut Block) -> u128;

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Category {
    #[default]
    Basic,
    InputOutput,
    Gate,
    Latch,
    FlipFlop,
    Hidden,
    Custom
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Custom {
    plot: Plot,
    input_block: BlockID,
    output_block: BlockID,
    cache: HashMap<u128, u128>
}

impl Custom {
    fn new(plot: Plot) -> Self {
        Self {
            plot,
            input_block: uuid::Uuid::default(),
            output_block: uuid::Uuid::default(),
            cache: HashMap::new()
        }
    }

    pub fn plot(&self) -> &Plot {
        &self.plot
    }

    pub fn plot_mut(&mut self) -> &mut Plot {
        &mut self.plot
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Module {
    name: String,
    category: Category,
    builtin: bool,
    num_inputs: u8,
    num_outputs: u8,
    decoration: Decoration,
    custom_data: Option<Custom>,
}

impl Module {
    pub const MAX_MODULE_NAME_LEN: i32 = 25;

    pub fn new(name: String, num_inputs: u8, num_outputs: u8) -> Self {
        Self {
            name,
            category: Category::Custom,
            builtin: false,
            custom_data: Some(Custom::new(Plot::new())),
            num_inputs,
            num_outputs,
            decoration: Decoration::None
        }
    }

    pub fn new_builtin<'a>(name: &'a str, category: Category, num_inputs: u8, num_outputs: u8, decoration: Decoration) -> Self {
        Self {
            name: name.to_string(),
            category,
            builtin: true,
            custom_data: None,
            num_inputs,
            num_outputs,
            decoration
        }
    }

    pub fn plot(&self) -> Option<&Plot> {
        match &self.custom_data {
            Some(data) => Some(data.plot()),
            None => None
        }
    }

    pub fn plot_mut(&mut self) -> Option<&mut Plot> {
        match &mut self.custom_data {
            Some(data) => Some(data.plot_mut()),
            None => None
        }
    }

    pub fn set_io_blocks(&mut self, input_block: BlockID, output_block: BlockID) {
        match &mut self.custom_data {
            Some(data) => {
                data.input_block = input_block;
                data.output_block = output_block
            }
            _ => ()
        }
    }

    pub fn builtin(&self) -> bool {
        self.builtin
    }

    pub fn hidden(&self) -> bool {
        matches!(self.category, Category::Hidden)
    }

    pub fn category(&self) -> Category {
        self.category
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

    pub fn simulate(&mut self, inputs: u128, instance: &mut Block, project: &mut Project, call_stack: &mut HashSet<String>) -> u128 {
        let outputs = 
        if self.builtin && let Some(builtin) = BUILTINS.get(self.name.as_str()) {
            builtin.simulate(inputs, instance)
        }
        else {
            if call_stack.contains(&self.name) {
                error!("recursion detected, {} is already on the call stack", self.name);
            }
            call_stack.insert(self.name.clone());

            let custom_data = self.custom_data.as_mut().expect("cannot simulate custom module without correct data");
            let plot = &mut custom_data.plot;

            instance.state().apply(plot);

            if let Some(input) = plot.get_block_mut(custom_data.input_block) {
                input.set_bytes(inputs);
            }

            plot.add_block_to_update(custom_data.input_block);
            plot.simulate(project, call_stack);
            
            if let Some(input) = plot.get_block_mut(custom_data.input_block) {
                input.set_bytes(0);
            }

            let outputs = plot.get_block(custom_data.output_block).map(|block| block.bytes()).unwrap_or(0);
            let state = PlotState::from(plot);
            instance.set_state(State::Inherit(state));

            call_stack.remove(&self.name);
            outputs
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