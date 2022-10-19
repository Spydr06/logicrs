pub mod builtin;

#[derive(Default, Debug)]
pub struct Module {
    name: String,
    num_inputs: i32,
    num_outputs: i32
}

impl Module {
    pub fn new(name: String, num_inputs: i32, num_outputs: i32) -> Self {
        Self {
            name,
            num_inputs,
            num_outputs
        }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_num_inputs(&self) -> i32 {
        self.num_inputs
    }

    pub fn get_num_outputs(&self) -> i32 {
        self.num_outputs
    }
}