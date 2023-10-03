use std::collections::HashMap;

use crate::simulator::{Category, Decoration};

use super::{Block, Module, SimulatorFn};

pub struct Builtin {
    module: Module,
    simulator_fn: SimulatorFn,
}

impl Builtin {
    pub fn new(module: Module, simulator_fn: SimulatorFn) -> Builtin {
        Self {
            module,
            simulator_fn,
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

        builtins.insert(
            "Low",
            Builtin::new(
                Module::new_builtin(
                    "Low",
                    Category::Basic,
                    0,
                    1,
                    Decoration::Label("0".to_string()),
                ),
                |_, _| 0,
            ),
        );

        builtins.insert(
            "High",
            Builtin::new(
                Module::new_builtin(
                    "High",
                    Category::Basic,
                    0,
                    1,
                    Decoration::Label("1".to_string()),
                ),
                |_, _| std::u128::MAX,
            ),
        );

        builtins.insert(
            "And",
            Builtin::new(
                Module::new_builtin(
                    "And",
                    Category::Gate,
                    2,
                    1,
                    Decoration::Label(String::from("&")),
                ),
                |input, _| (input == 0b11) as u128,
            ),
        );

        builtins.insert(
            "Nand",
            Builtin::new(
                Module::new_builtin(
                    "Nand",
                    Category::Gate,
                    2,
                    1,
                    Decoration::NotLabel(String::from("&")),
                ),
                |input, _| (input != 0b11) as u128,
            ),
        );

        builtins.insert(
            "Or",
            Builtin::new(
                Module::new_builtin(
                    "Or",
                    Category::Gate,
                    2,
                    1,
                    Decoration::Label(String::from("≥1")),
                ),
                |input, _| (input != 0b00) as u128,
            ),
        );

        builtins.insert(
            "Nor",
            Builtin::new(
                Module::new_builtin(
                    "Nor",
                    Category::Gate,
                    2,
                    1,
                    Decoration::NotLabel(String::from("≥1")),
                ),
                |input, _| (input == 0b00) as u128,
            ),
        );

        builtins.insert(
            "Not",
            Builtin::new(
                Module::new_builtin(
                    "Not",
                    Category::Gate,
                    1,
                    1,
                    Decoration::NotLabel(String::from("1")),
                ),
                |input, _| !input,
            ),
        );

        builtins.insert(
            "Xor",
            Builtin::new(
                Module::new_builtin(
                    "Xor",
                    Category::Gate,
                    2,
                    1,
                    Decoration::Label(String::from("=1")),
                ),
                |input, _| (input == 0b01 || input == 0b10) as u128,
            ),
        );

        builtins.insert(
            "Xnor",
            Builtin::new(
                Module::new_builtin(
                    "Xnor",
                    Category::Gate,
                    2,
                    1,
                    Decoration::NotLabel(String::from("=1")),
                ),
                |input, _| (input == 0b00 || input == 0b11) as u128,
            ),
        );

        builtins.insert(
            "Button",
            Builtin::new(
                Module::new_builtin(
                    "Button",
                    Category::InputOutput,
                    0,
                    1,
                    Decoration::Button(false),
                ),
                |_, instance| instance.is_active() as u128,
            ),
        );

        builtins.insert(
            "Switch",
            Builtin::new(
                Module::new_builtin(
                    "Switch",
                    Category::InputOutput,
                    0,
                    1,
                    Decoration::Switch(false),
                ),
                |_, instance| instance.is_active() as u128,
            ),
        );

        builtins.insert(
            "Lamp",
            Builtin::new(
                Module::new_builtin("Lamp", Category::InputOutput, 1, 0, Decoration::Lamp(false)),
                |input, instance| {
                    instance.set_active(input & 0b01 == 0b01);
                    0
                },
            ),
        );

        builtins.insert(
            "Input",
            Builtin::new(
                Module::new_builtin(
                    "Input",
                    Category::Hidden,
                    Block::MAX_CONNECTIONS,
                    Block::MAX_CONNECTIONS,
                    Decoration::Label("|>".to_string()),
                ),
                |input, instance| {
                    if instance.passthrough() {
                        input
                    } else {
                        instance.bytes()
                    }
                },
            ),
        );

        builtins.insert(
            "Output",
            Builtin::new(
                Module::new_builtin(
                    "Output",
                    Category::Hidden,
                    Block::MAX_CONNECTIONS,
                    Block::MAX_CONNECTIONS,
                    Decoration::Label(">|".to_string()),
                ),
                |input, instance| {
                    instance.set_bytes(input);
                    input
                },
            ),
        );

        builtins.insert(
            "Mux",
            Builtin::new(
                Module::new_builtin(
                    "Mux",
                    Category::Combinational,
                    3,
                    1,
                    Decoration::Label(String::from("Mux")),
                ),
                |input, _| (input & 0b101 == 0b001 || input & 0b110 == 0b110) as u128,
            ),
        );

        builtins.insert(
            "Demux",
            Builtin::new(
                Module::new_builtin(
                    "Demux",
                    Category::Combinational,
                    2,
                    2,
                    Decoration::Label(String::from("Demux")),
                ),
                |input, _| ((((input == 0b11) as u128) << 1) | (input == 0b01) as u128),
            ),
        );

        builtins.insert(
            "SR Nand Latch",
            Builtin::new(
                Module::new_builtin(
                    "SR Nand Latch",
                    Category::Latch,
                    2,
                    2,
                    Decoration::NotLabel("SR".to_string()),
                ),
                sr_nand_latch,
            ),
        );

        builtins.insert(
            "SR Latch",
            Builtin::new(
                Module::new_builtin(
                    "SR Latch",
                    Category::Latch,
                    2,
                    1,
                    Decoration::Label("SR".to_string()),
                ),
                sr_latch,
            ),
        );

        builtins.insert(
            "JK Latch",
            Builtin::new(
                Module::new_builtin(
                    "JK Latch",
                    Category::Latch,
                    2,
                    1,
                    Decoration::Label("JK".to_string()),
                ),
                jk_latch,
            ),
        );

        builtins.insert(
            "T Flip-Flop",
            Builtin::new(
                Module::new_builtin(
                    "T Flip-Flop",
                    Category::FlipFlop,
                    1,
                    1,
                    Decoration::Label("T".to_string()),
                ),
                t_flip_flop,
            ),
        );

        builtins
    };
}

fn jk_latch(input: u128, instance: &mut Block) -> u128 {
    let j = input & 0b01 > 0;
    let k = input & 0b10 > 0;

    if j && k {
        instance.set_bytes((instance.bytes() == 0) as u128);
    } else if j {
        instance.set_bytes(1);
    } else if k {
        instance.set_bytes(0);
    }

    instance.bytes()
}

fn sr_latch(input: u128, instance: &mut Block) -> u128 {
    // reset (R)
    if input & 0b10 > 0 {
        instance.set_bytes(0);
    }
    // set (S)
    else if input & 0b01 > 0 {
        instance.set_bytes(1);
    }

    instance.bytes()
}

fn sr_nand_latch(input: u128, instance: &mut Block) -> u128 {
    // set (S)
    if input & 0b01 == 0 {
        instance.set_bytes(1);
    }
    // reset (R)
    else if input & 0b10 == 0 {
        instance.set_bytes(0);
    }

    instance.bytes() | (((instance.bytes() == 0) as u128) << 1)
}

fn t_flip_flop(input: u128, instance: &mut Block) -> u128 {
    if input & 1 > 0 && instance.bytes() & 0b10 == 0 {
        instance.set_bytes(instance.bytes() ^ 1);
    }
    instance.set_bytes((instance.bytes() & !0b10) | (input << 1));
    instance.bytes() & 1
}
