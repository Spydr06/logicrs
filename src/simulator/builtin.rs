use crate::simulator::Decoration;

use super::Module;

lazy_static! {
   pub static ref INPUT_MODULE_NAME: String = String::from("Input");
   pub static ref OUTPUT_MODULE_NAME: String = String::from("Output");
   pub static ref BUILTINS: Vec<Module> = {
      vec![
         Module::new_builtin("And",    false, 2, 1, Decoration::Label(String::from("&"))),
         Module::new_builtin("Nand",   false, 2, 1, Decoration::NotLabel(String::from("&"))),
         Module::new_builtin("Or",     false, 2, 1, Decoration::Label(String::from("≥1"))),
         Module::new_builtin("Nor",    false, 2, 1, Decoration::NotLabel(String::from("≥1"))),
         Module::new_builtin("Not",    false, 1, 1, Decoration::NotLabel(String::from("1"))),
         Module::new_builtin("Xor",    false, 2, 1, Decoration::Label(String::from("=1"))),
         Module::new_builtin("Xnor",    false, 2, 1, Decoration::NotLabel(String::from("=1"))),
         Module::new_builtin("Button", false, 0, 1, Decoration::None),
         Module::new_builtin("Switch", false, 0, 1, Decoration::None),
         Module::new_builtin("Lamp",   false, 1, 0, Decoration::None),

         // Hidden modules, only used for custom module IO
         Module::new_builtin(&INPUT_MODULE_NAME,  true , 0, std::u8::MAX, Decoration::None),
         Module::new_builtin(&OUTPUT_MODULE_NAME, true , std::u8::MAX, 0, Decoration::None)
      ]
   };
}
