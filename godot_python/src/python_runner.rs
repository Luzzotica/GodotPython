use std::collections::HashMap;

use godot::prelude::*;
use rustpython_vm::{self as vm, import::import_source, TryFromObject};

#[derive(GodotClass)]
#[class(base=Node)]
pub struct PythonRunner {
    interpreter: vm::Interpreter,
    modules: HashMap<String, vm::PyObjectRef>,
    node: Base<Node>,
}

#[godot_api]
impl INode for PythonRunner {
    fn init(node: Base<Node>) -> Self {
        let interp = vm::Interpreter::without_stdlib(Default::default());

        // Add stdout function

        Self {
            node,
            interpreter: interp,
            modules: HashMap::new(),
        }
    }
}

#[godot_api]
impl PythonRunner {
    #[func]
    fn run_python(&mut self, code: String) -> Variant {
        // godot_print!("Running Python code: {}", code);

        self.interpreter.enter(|virt| {
            let scope = virt.new_scope_with_builtins();
            let output = virt.run_block_expr(scope, &code);
            // godot_print!("Output: {:?}", output);

            match output {
                Ok(value) => {
                    // Handle success case
                    // godot_print!("Success: {:?}", value);
                    if let Ok(int_obj) = i64::try_from_object(virt, value.clone()) {
                        Variant::from(int_obj)
                    } else if let Ok(float_obj) = f64::try_from_object(virt, value.clone()) {
                        Variant::from(float_obj)
                    } else if let Ok(str_obj) = String::try_from_object(virt, value.clone()) {
                        Variant::from(str_obj)
                    } else {
                        Variant::from("Unknown type")
                    }
                }
                Err(error) => {
                    // Handle error case
                    godot_print!("Error: {:?}", error);
                    Variant::from("Error")
                }
            }
        })
    }

    #[func]
    fn load_module(&mut self, module_name: String, module_code: String) -> Variant {
        // godot_print!("Loading module: {}", module_code);

        self.interpreter.enter(|virt| {
            let result = import_source(virt, &module_name, &module_code);
            // godot_print!("Result: {:?}", result);

            match result {
                Ok(value) => {
                    // Handle success case
                    // godot_print!("Success: {:?}", value);
                    self.modules.insert(module_name, value);
                    Variant::from("Success")
                }
                Err(error) => {
                    // Handle error case
                    // godot_print!("Error: {:?}", error);
                    Variant::from(format!("Error: {:?}", error))
                }
            }
        })
    }

    #[func]
    fn input_down(&mut self, module_name: String, i: String) -> String {
        // let module_name_clone = &module_name.clone();
        let module = self.modules.get(&module_name).unwrap();

        self.interpreter.enter(move |virt| {
            // let scope = virt.new_scope_with_builtins();
            let exec_fn = module.get_attr("input_down", virt).unwrap();

            let result = exec_fn.call((i,), virt);

            match result {
                Ok(value) => {
                    // Handle success case
                    // godot_print!("Success: {:?}", value);

                    if let Ok(str_obj) = String::try_from_object(virt, value) {
                        str_obj
                    } else {
                        "Error".to_string()
                    }
                }
                Err(error) => {
                    // Handle error case
                    godot_print!("Error: {:?}", error);
                    "Error".to_string()
                }
            }
        })

        // Variant::from("Success")
    }

    #[func]
    fn input_up(&mut self, module_name: String, i: String) -> String {
        // let module_name_clone = &module_name.clone();
        let module = self.modules.get(&module_name).unwrap();

        self.interpreter.enter(move |virt| {
            // let scope = virt.new_scope_with_builtins();
            let exec_fn = module.get_attr("input_up", virt).unwrap();

            let result = exec_fn.call((i,), virt);

            match result {
                Ok(value) => {
                    // Handle success case
                    // godot_print!("Success: {:?}", value);

                    if let Ok(str_obj) = String::try_from_object(virt, value) {
                        str_obj
                    } else {
                        "Error".to_string()
                    }
                }
                Err(error) => {
                    // Handle error case
                    godot_print!("Error: {:?}", error);
                    "Error".to_string()
                }
            }
        })

        // Variant::from("Success")
    }
}
