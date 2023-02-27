use std::collections::HashMap;

use crate::simulator::Decoration;

use super::{Module, SimulatorFn, Block};

pub struct Builtin {
    module: Module,
    simulator_fn: SimulatorFn
}

impl Builtin {
    pub fn new(module: Module, simulator_fn: SimulatorFn) -> Builtin {
        Self {
            module,
            simulator_fn
        }
    }

    pub fn module(&self) -> &Module {
        &self.module
    }

    pub fn simulate(&self, inputs: u128, instance: &mut Block) -> u128 {
        (self.simulator_fn)(inputs, instance)
    }
}

lazy_static! {
    pub static ref INPUT_MODULE_NAME: String = String::from("Input");
    pub static ref OUTPUT_MODULE_NAME: String = String::from("Output");
    pub static ref BUILTINS: HashMap<&'static str, Builtin> = {
        let mut builtins = HashMap::new();

        builtins.insert("And", Builtin::new(
            Module::new_builtin("And", false, 2, 1, Decoration::Label(String::from("&"))),
            |input, _| (input == 0b11) as u128
        ));

        builtins.insert("Nand", Builtin::new(
            Module::new_builtin("Nand", false, 2, 1, Decoration::NotLabel(String::from("&"))),
            |input, _| (input & 0b10 != 0b10) as u128
        ));

        builtins.insert("Or", Builtin::new(
            Module::new_builtin("Or", false, 2, 1, Decoration::Label(String::from("≥1"))),
            |input, _| (input > 0) as u128
        ));

        builtins.insert("Nor", Builtin::new(
            Module::new_builtin("Nor", false, 2, 1, Decoration::NotLabel(String::from("≥1"))),
            |input, _| (input == 0) as u128
        ));

        builtins.insert("Not", Builtin::new(
            Module::new_builtin("Not", false, 1, 1, Decoration::NotLabel(String::from("1"))),
            |input, _| !input
        ));

        builtins.insert("Xor", Builtin::new(
            Module::new_builtin("Xor", false, 2, 1, Decoration::Label(String::from("=1"))),
            |input, _| (input == 0b01 || input == 0x10) as u128
        ));

        builtins.insert("Xnor", Builtin::new(
            Module::new_builtin("Xnor", false, 2, 1, Decoration::NotLabel(String::from("=1"))),
            |input, _| (input == 0b00 || input == 0b10) as u128
        ));

        builtins.insert("Button", Builtin::new(
            Module::new_builtin("Button", false, 0, 1, Decoration::Button(false)),
            |_, instance| instance.is_active() as u128
        ));

        builtins.insert("Switch", Builtin::new(
            Module::new_builtin("Switch", false, 0, 1, Decoration::Switch(false)),
            |_, instance| instance.is_active() as u128
        ));

        builtins.insert("Lamp", Builtin::new(
            Module::new_builtin("Lamp", false, 1, 0, Decoration::Lamp(false)),
            |input, instance| { 
                instance.set_active(input & 0b01 == 0b01);
                0
            }
        ));

        builtins.insert("Input", Builtin::new(
            Module::new_builtin("Input", true, 0, Block::MAX_CONNECTIONS, Decoration::None),
            |_, _| { 0 }
        ));

        builtins.insert("Output", Builtin::new(
            Module::new_builtin("Output", true, Block::MAX_CONNECTIONS, 0, Decoration::None),
            |_, _| { 0 }
        ));
        
        builtins
    };
}
