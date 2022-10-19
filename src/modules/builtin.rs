use super::Module;

pub fn register(data: &mut crate::application::data::ApplicationData) {
   data.add_module(Module::new("And".to_string(), 2, 1));
   data.add_module(Module::new("Or".to_string(), 2, 1));
   data.add_module(Module::new("Not".to_string(), 1, 1));
   data.add_module(Module::new("Xor".to_string(), 2, 1));
}