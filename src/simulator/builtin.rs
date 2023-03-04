use std::collections::HashMap;

use crate::simulator::{Decoration, Category};

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

        builtins.insert("Low", Builtin::new(
            Module::new_builtin("Low", Category::Basic, 0, 1, Decoration::Label("0".to_string())),
            |_, _| { 0 }
        ));

        builtins.insert("High", Builtin::new(
            Module::new_builtin("High", Category::Basic, 0, 1, Decoration::Label("1".to_string())),
            |_, _| { std::u128::MAX }
        ));

        builtins.insert("Junction", Builtin::new(
            Module::new_builtin("Junction", Category::Basic, 1, 2, Decoration::None),
            |input, _| [0, std::u128::MAX][input as usize]
        ));

        builtins.insert("And", Builtin::new(
            Module::new_builtin("And", Category::Gate, 2, 1, Decoration::Label(String::from("&"))),
            |input, _| (input == 0b11) as u128
        ));

        builtins.insert("Nand", Builtin::new(
            Module::new_builtin("Nand", Category::Gate, 2, 1, Decoration::NotLabel(String::from("&"))),
            |input, _| (input & 0b10 != 0b10) as u128
        ));

        builtins.insert("Or", Builtin::new(
            Module::new_builtin("Or", Category::Gate, 2, 1, Decoration::Label(String::from("≥1"))),
            |input, _| (input > 0) as u128
        ));

        builtins.insert("Nor", Builtin::new(
            Module::new_builtin("Nor", Category::Gate, 2, 1, Decoration::NotLabel(String::from("≥1"))),
            |input, _| (input == 0) as u128
        ));

        builtins.insert("Not", Builtin::new(
            Module::new_builtin("Not", Category::Gate, 1, 1, Decoration::NotLabel(String::from("1"))),
            |input, _| !input
        ));

        builtins.insert("Xor", Builtin::new(
            Module::new_builtin("Xor", Category::Gate, 2, 1, Decoration::Label(String::from("=1"))),
            |input, _| (input == 0b01 || input == 0b10) as u128
        ));

        builtins.insert("Xnor", Builtin::new(
            Module::new_builtin("Xnor", Category::Gate, 2, 1, Decoration::NotLabel(String::from("=1"))),
            |input, _| (input == 0b00 || input == 0b10) as u128
        ));

        builtins.insert("Button", Builtin::new(
            Module::new_builtin("Button", Category::InputOutput, 0, 1, Decoration::Button(false)),
            |_, instance| instance.is_active() as u128
        ));

        builtins.insert("Switch", Builtin::new(
            Module::new_builtin("Switch", Category::InputOutput, 0, 1, Decoration::Switch(false)),
            |_, instance| instance.is_active() as u128
        ));

        builtins.insert("Lamp", Builtin::new(
            Module::new_builtin("Lamp", Category::InputOutput, 1, 0, Decoration::Lamp(false)),
            |input, instance| { 
                instance.set_active(input & 0b01 == 0b01);
                0
            }
        ));

        builtins.insert("Input", Builtin::new(
            Module::new_builtin("Input", Category::Hidden, 0, Block::MAX_CONNECTIONS, Decoration::None),
            |_, instance| { instance.state() }
        ));

        builtins.insert("Output", Builtin::new(
            Module::new_builtin("Output", Category::Hidden, Block::MAX_CONNECTIONS, 0, Decoration::None),
            |input, instance| { instance.set_state(input); 0 }
        ));
        
        builtins
    };
}
