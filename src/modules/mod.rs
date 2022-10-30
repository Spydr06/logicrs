pub mod builtin;

#[derive(Default, Debug)]
pub struct Module {
    name: String,
    num_inputs: u8,
    num_outputs: u8
}

impl Module {
    pub fn new(name: String, num_inputs: u8, num_outputs: u8) -> Self {
        Self {
            name,
            num_inputs,
            num_outputs
        }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_num_inputs(&self) -> u8 {
        self.num_inputs
    }

    pub fn get_num_outputs(&self) -> u8 {
        self.num_outputs
    }
}