use super::Module;

lazy_static! {
   pub static ref BUILTINS: Vec<Module> = {
      vec![
         Module::new_builtin("And", 2, 1),
         Module::new_builtin("Or", 2, 1),
         Module::new_builtin("Not", 1, 1),
         Module::new_builtin("Xor", 2, 1),
         Module::new_builtin("Button", 0, 1),
         Module::new_builtin("Switch", 0, 1)
      ]
   };
}
