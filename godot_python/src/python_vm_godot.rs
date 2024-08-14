mod godot_converter;

use godot::prelude::*;
use godot_converter::{
    convert_py_to_variant_common, convert_variant_arr_to_args, convert_variant_dict_to_kwargs,
};
use rustpython_vm::function::FuncArgs;

use crate::python_vm_common::CommonPythonVM;

#[derive(GodotClass)]
#[class(base=Node)]
pub struct GodotPythonVM {
    node: Base<Node>,
    common_vm: CommonPythonVM,
}

#[godot_api]
impl INode for GodotPythonVM {
    fn init(node: Base<Node>) -> Self {
        let common_vm = CommonPythonVM::init();

        Self { node, common_vm }
    }
}

#[godot_api]
impl GodotPythonVM {
    #[func]
    fn setup_stdout(&mut self, godot_out: Gd<Node>) {
        let callable = godot_out.callable("append_output");

        let clo = move |s: String| {
            let mut arr = VariantArray::new();
            arr.push(Variant::from(s));
            callable.callv(arr);
        };

        self.common_vm.setup_stdout(clo);
    }

    #[func]
    fn eval(&self, code: String) -> Variant {
        let r = self.common_vm.eval(code);

        match r {
            Ok(value) => convert_py_to_variant_common(&self.common_vm, value),
            Err(error) => Variant::from(format!("Error: {:?}", error)),
        }
    }

    #[func]
    fn load_module(&mut self, module_name: String, module_code: String) -> Variant {
        let r = self.common_vm.load_module(module_name.clone(), module_code);

        match r {
            Ok(_) => Variant::from("Success"),
            Err(error) => Variant::from(format!("Error: {:?}", error)),
        }
    }

    #[func]
    fn call_python_function(
        &self,
        module_name: String,
        function_name: String,
        args: VariantArray,
        kwargs: Dictionary,
    ) -> Variant {
        let f_args = FuncArgs::new(
            convert_variant_arr_to_args(&self.common_vm, args),
            convert_variant_dict_to_kwargs(&self.common_vm, kwargs),
        );
        let r = self
            .common_vm
            .call_python_function(module_name, function_name, f_args);

        match r {
            Ok(value) => convert_py_to_variant_common(&self.common_vm, value),
            Err(error) => Variant::from(format!("Error: {:?}", error)),
        }
    }
}
