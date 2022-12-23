use super::Module;

pub fn register(data: &mut crate::application::data::ApplicationData) {
   data.add_module(Module::new_builtin("And".to_string(), 2, 1));
   data.add_module(Module::new_builtin("Or".to_string(), 2, 1));
   data.add_module(Module::new_builtin("Not".to_string(), 1, 1));
   data.add_module(Module::new_builtin("Xor".to_string(), 2, 1));
   data.add_module(Module::new_builtin("Button".to_string(), 0, 1));
   data.add_module(Module::new_builtin("Switch".to_string(), 0, 1));
}