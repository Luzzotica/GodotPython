pub mod python_converter;
pub mod rust_stdout;

use std::collections::HashMap;

use python_converter::unwrap_error;
use rust_stdout::{create_rust_stdout, rust_stdout::RustStdout};
use rustpython_vm::{
    builtins::{PyStr, PyStrRef},
    convert::ToPyObject,
    function::FuncArgs,
    import::import_source,
    Interpreter, PyObjectRef, PyPayload,
};

pub struct CommonPythonVM {
    pub interpreter: Interpreter,
    modules: HashMap<String, PyObjectRef>,
}

impl CommonPythonVM {
    pub fn init() -> Self {
        let interpreter = rustpython::InterpreterConfig::new()
            .init_stdlib()
            .init_hook(Box::new(|vm| {
                vm.add_native_module("rust_stdout".to_owned(), create_rust_stdout());
            }))
            .interpreter();

        let stdout_override_module = interpreter.enter(|vm| {
            let r = import_source(
                vm,
                "stdout_override",
                r#"
import sys
from rust_stdout import rs_print, rs_print_err

class CustomStream:

  def __init__(self, callback, start="", end=""):
    self.callback = callback
    self.buffer = start
    self.start = start
    self.end = end


  def write(self, message):
    self.buffer += message
    if "\n" == message:
      self.buffer += self.end
      self.flush()

        
  def flush(self):
    if self.buffer:
      self.callback(self.buffer)
      self.buffer = self.start


def set_rust_stdout(rstd):
  # Set the custom stream as the new stdout
  sys.stdout = CustomStream(rstd.rs_print)
  sys.stderr = CustomStream(
    rstd.rs_print_err, start="[color=red]", end="[/color]"
  )


sys.stdout = CustomStream(rs_print)
sys.stderr = CustomStream(rs_print_err, start="[color=red]", end="[/color]")
"#,
            );

            // if let Err(e) = r {
            //     print!("Error: {:?}", e);
            // }

            match r {
                Ok(value) => value,
                Err(error) => {
                    let mut s = "Error: ".to_string();
                    error.args().into_iter().for_each({
                        |v| {
                            s.push_str(&format!("{:?}\n", v));
                        }
                    });
                    print!("{}", s);
                    vm.ctx.new_str(s).to_pyobject(vm)
                }
            }
        });

        let mut modules: HashMap<String, PyObjectRef> = HashMap::new();
        modules.insert("stdout_override".to_owned(), stdout_override_module);

        Self {
            interpreter,
            modules,
        }
    }

    pub fn setup_stdout<T>(&mut self, closure: T)
    where
        T: Fn(String) + 'static,
    {
        let mut rust_stdout = RustStdout::new();
        rust_stdout.set_stdout_fn(Box::new(closure));

        // rust_stdout.set_stderr_fn(Box::new(|s| {
        //     self.std_err(s);
        // }));

        self.interpreter.enter(|vm| {
            let f = self
                .modules
                .get("stdout_override")
                .unwrap()
                .get_attr("set_rust_stdout", vm);

            let rstd_pyobj = rust_stdout.to_pyobject(vm);

            match f {
                Ok(f) => {
                    let _ = f.call((rstd_pyobj,), vm);
                }
                Err(error) => {
                    let _ =
                        rstd_pyobj.call((format!("Error setting rust_stdout: {:?}", error),), vm);
                }
            }
        });
    }

    pub fn eval(&self, code: String) -> Result<PyObjectRef, String> {
        self.interpreter.enter(|vm| {
            let scope = vm.new_scope_with_builtins();
            let output = vm.run_block_expr(scope, &code);
            // godot_print!("Output: {:?}", output);

            match output {
                Ok(value) => Ok(value),
                Err(error) => Err(unwrap_error(vm, error)),
            }
        })
    }

    pub fn load_module(
        &mut self,
        module_name: String,
        module_code: String,
    ) -> Result<PyObjectRef, String> {
        self.interpreter.enter(|vm| {
            let result = import_source(vm, &module_name, &module_code);
            // godot_print!("Result: {:?}", result);

            match result {
                Ok(value) => {
                    self.modules.insert(module_name, value.clone());
                    Ok(value)
                }
                Err(error) => Err(unwrap_error(vm, error)),
            }
        })
    }

    pub fn call_python_function(
        &self,
        module_name: String,
        function_name: String,
        f_args: FuncArgs,
    ) -> Result<PyObjectRef, String> {
        self.interpreter.enter(|vm| {
            let module_r = self.modules.get(&module_name);
            let module = match module_r {
                Some(m) => m,
                None => {
                    return Err(format!("Error: Module not found: {:?}", module_name));
                }
            };

            let attr_name: PyStrRef = PyStr::from(function_name).into_ref(&vm.ctx);
            // let scope = virt.new_scope_with_builtins();
            let exec_fn = module.get_attr(&attr_name, vm).unwrap();

            let result = exec_fn.call_with_args(f_args, vm);

            match result {
                Ok(value) => Ok(value),
                Err(error) => Err(unwrap_error(vm, error)),
            }

            // Variant::from("Success")
        })
    }
}

#[cfg(test)]
pub mod tests {
    use rustpython_vm::{builtins::PyInt, function::KwArgs, TryFromObject};
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    #[test]
    fn test_init() {
        test_init_common()
    }
    #[wasm_bindgen_test]
    fn test_init_web() {
        test_init_common()
    }
    fn test_init_common() {
        let common_vm = CommonPythonVM::init();

        // print!("{:?}", stdout);
        // assert_eq!(
        //     format!("{:?}", stdout),
        //     "[PyObject PyModule { def: None, name: None }]"
        // );

        let r = common_vm
            .eval(
                r#"
from rust_stdout import rs_print
rs_print
"#
                .to_string(),
            )
            .unwrap();
        assert_eq!(format!("{:?}", r), "[PyObject builtin function rust_stdout.rs_print (PyMethodFlags(0x0)) self as instance of None]");

        let r2 = common_vm
            .eval(
                r#"
from stdout_override import CustomStream
CustomStream
"#
                .to_string(),
            )
            .unwrap();
        assert_eq!(format!("{:?}", r2), "[PyObject [PyType CustomStream]]");
    }

    #[test]
    fn test_eval() {
        test_eval_common()
    }
    #[wasm_bindgen_test]
    fn test_eval_web() {
        test_eval_common()
    }
    fn test_eval_common() {
        let common_vm = CommonPythonVM::init();
        let r = common_vm.eval("'Hello, World!'".to_string()).unwrap();
        common_vm.interpreter.enter(|vm| {
            assert_eq!(String::try_from_object(vm, r).unwrap(), "Hello, World!");
        });
    }

    #[test]
    fn test_load_module() {
        test_load_module_common()
    }
    #[wasm_bindgen_test]
    fn test_load_module_web() {
        test_load_module_common()
    }
    fn test_load_module_common() {
        let mut common_vm = CommonPythonVM::init();
        let _ = common_vm
            .load_module(
                "test_module".to_string(),
                r#"
def hello():
  return "Hello, World!2"
        "#
                .to_string(),
            )
            .unwrap();
        let hello = common_vm
            .eval(
                r#"from test_module import hello
hello()"#
                    .to_string(),
            )
            .unwrap();
        common_vm.interpreter.enter(|vm| {
            assert_eq!(
                String::try_from_object(vm, hello).unwrap(),
                "Hello, World!2"
            );
        });
    }

    #[test]
    fn test_call_python_function() {
        test_call_python_function_common()
    }
    #[wasm_bindgen_test]
    fn test_call_python_function_web() {
        test_call_python_function_common()
    }
    fn test_call_python_function_common() {
        let mut common_vm = CommonPythonVM::init();
        let _ = common_vm
            .load_module(
                "test_module".to_string(),
                r#"
def hello():
  return "Hello, World!"

def hello_args(s):
  return "Hello, " + s

def hello_kwargs(s, n=1):
  return "Hello, " + s + ". Count: " + str(n)
        "#
                .to_string(),
            )
            .unwrap();

        common_vm.interpreter.enter(|vm| {
            let hello = common_vm
                .call_python_function(
                    "test_module".to_string(),
                    "hello".to_string(),
                    FuncArgs::default(),
                )
                .unwrap();
            assert_eq!(String::try_from_object(vm, hello).unwrap(), "Hello, World!");

            let hello_args = common_vm
                .call_python_function(
                    "test_module".to_string(),
                    "hello_args".to_string(),
                    FuncArgs::new(
                        vec![PyStr::from("Sterling".to_string()).to_pyobject(vm)],
                        KwArgs::default(),
                    ),
                )
                .unwrap();
            assert_eq!(
                String::try_from_object(vm, hello_args).unwrap(),
                "Hello, Sterling"
            );

            let hello_kwargs = common_vm
                .call_python_function(
                    "test_module".to_string(),
                    "hello_kwargs".to_string(),
                    FuncArgs::new(
                        vec![PyStr::from("Sterling".to_string()).to_pyobject(vm)],
                        KwArgs::new(
                            vec![("n".to_string(), PyInt::from(1).to_pyobject(vm))]
                                .into_iter()
                                .collect(),
                        ),
                    ),
                )
                .unwrap();
            assert_eq!(
                String::try_from_object(vm, hello_kwargs).unwrap(),
                "Hello, Sterling. Count: 1"
            );
        });
    }

    // #[test]
    // fn test_load_module() {
    //     let (interp, _) = create_interpreter();
    //     interp.enter(|vm| {
    //         let r = load_module(
    //             vm,
    //             "test_module".to_string(),
    //             r#"
}
