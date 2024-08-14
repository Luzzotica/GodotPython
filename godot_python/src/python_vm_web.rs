mod wasm_converter;

use std::panic;

use js_sys::{Array, Object, Reflect, WebAssembly::RuntimeError};
use rustpython_vm::function::FuncArgs;
use wasm_bindgen::prelude::*;
use wasm_converter::{convert_js_arr_to_args, convert_js_obj_to_kwargs, convert_py_to_js_common};
use web_sys::console;

use crate::python_vm_common::CommonPythonVM;

/// Sets error info on the window object, and prints the backtrace to console
pub fn panic_hook(info: &panic::PanicInfo) {
    // If something errors, just ignore it; we don't want to panic in the panic hook
    let try_set_info = || {
        let msg = &info.to_string();
        let window = match web_sys::window() {
            Some(win) => win,
            None => return,
        };
        let _ = Reflect::set(&window, &"__RUSTPYTHON_ERROR_MSG".into(), &msg.into());
        let error = RuntimeError::new(msg);
        let _ = Reflect::set(&window, &"__RUSTPYTHON_ERROR".into(), &error);
        let stack = match Reflect::get(&error, &"stack".into()) {
            Ok(stack) => stack,
            Err(_) => return,
        };
        let _ = Reflect::set(&window, &"__RUSTPYTHON_ERROR_STACK".into(), &stack);
    };
    try_set_info();
    console_error_panic_hook::hook(info);
}

#[doc(hidden)]
#[wasm_bindgen(start)]
pub fn _setup_console_error() {
    std::panic::set_hook(Box::new(panic_hook));
}

#[wasm_bindgen(js_name = WasmPythonVM)]
pub struct WasmPythonVM {
    common_vm: CommonPythonVM,
}

#[wasm_bindgen(js_class = WasmPythonVM)]
impl WasmPythonVM {
    #[wasm_bindgen(constructor)]
    pub fn init() -> Self {
        let common_vm = CommonPythonVM::init();

        Self { common_vm }
    }

    #[wasm_bindgen]
    pub fn setup_stdout(&mut self, callable: JsValue) {
        // self.godot_stdout = Some(godot_out);

        let function_r = callable.dyn_into::<js_sys::Function>();

        let function = match function_r {
            Ok(f) => f,
            Err(e) => {
                console::log_1(&"Error getting function:".into());
                console::log_1(&e);
                return;
            }
        };

        let clo = move |s: String| {
            let r = function.call1(&JsValue::NULL, &s.as_str().into());
            match r {
                Ok(_) => {}
                Err(e) => {
                    console::log_1(&"Error calling function:".into());
                    console::log_1(&e);
                }
            }
        };

        self.common_vm.setup_stdout(clo);
    }

    #[wasm_bindgen]
    pub fn eval(&self, code: String) -> JsValue {
        let r = self.common_vm.eval(code);

        match r {
            Ok(value) => convert_py_to_js_common(&self.common_vm, value),
            Err(error) => JsValue::from(format!("Error: {:?}", error)),
        }
    }

    #[wasm_bindgen]
    pub fn load_module(&mut self, module_name: String, module_code: String) -> JsValue {
        let r = self.common_vm.load_module(module_name.clone(), module_code);

        match r {
            Ok(_) => JsValue::from("Success"),
            Err(error) => JsValue::from(format!("Error: {:?}", error)),
        }
    }

    #[wasm_bindgen]
    pub fn call_python_function(
        &mut self,
        module_name: String,
        function_name: String,
        args: Array,
        kwargs: Object,
    ) -> JsValue {
        let f_args = FuncArgs::new(
            convert_js_arr_to_args(&self.common_vm, args),
            convert_js_obj_to_kwargs(&self.common_vm, kwargs),
        );
        let r = self
            .common_vm
            .call_python_function(module_name, function_name, f_args);

        match r {
            Ok(value) => convert_py_to_js_common(&self.common_vm, value),
            Err(error) => JsValue::from(format!("Error: {:?}", error)),
        }
    }
}
