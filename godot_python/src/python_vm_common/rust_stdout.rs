use rustpython_vm::{builtins::PyModule, pymodule, PyRef, VirtualMachine};

pub fn create_rust_stdout() -> Box<fn(&VirtualMachine) -> PyRef<PyModule>> {
    Box::new(rust_stdout::make_module)
}

#[pymodule]
pub mod rust_stdout {

    use std::fmt;
    use std::fmt::Formatter;

    use super::*;
    #[cfg(not(target_arch = "wasm32"))]
    use godot::log::{godot_error, godot_print};
    use rustpython_vm::{pyclass, PyPayload};
    #[cfg(target_arch = "wasm32")]
    use web_sys::{self, console};

    #[pyattr]
    #[pyclass(module = "rust_stdout", name = "RustStdout")]
    #[derive(Debug, PyPayload)]
    pub struct RustStdout {
        fns: RustStdoutFns,
    }

    #[pyclass]
    impl RustStdout {
        pub fn new() -> Self {
            Self {
                fns: RustStdoutFns::new(),
            }
        }

        pub fn set_stdout_fn(&mut self, f: Box<dyn Fn(String)>) {
            self.fns.set_stdout_fn(f);
        }

        pub fn set_stderr_fn(&mut self, f: Box<dyn Fn(String)>) {
            self.fns.set_stderr_fn(f);
        }

        #[pymethod]
        fn rs_print(&self, args: String) {
            if let Some(f) = &self.fns.stdout_fn {
                f(args);
            } else {
                println!("{}", args);
            }
        }

        #[pymethod]
        fn rs_print_err(&self, args: String) {
            if let Some(f) = &self.fns.stderr_fn {
                f(args);
            } else {
                println!("{}", args);
            }
        }
    }

    #[pyfunction]
    fn rs_print(args: String, _vm: &VirtualMachine) {
        cfg_if::cfg_if! {
            if #[cfg(not(target_arch = "wasm32"))] {
                godot_print!("{}", args);
            }
            else if #[cfg(target_arch = "wasm32")] {
                console::log_1(&args.as_str().into());
            }
        }
    }

    #[pyfunction]
    fn rs_print_err(args: String, _vm: &VirtualMachine) {
        cfg_if::cfg_if! {
            if #[cfg(not(target_arch = "wasm32"))] {
                godot_error!("{}", args);
            }
            else if #[cfg(target_arch = "wasm32")] {
                console::error_1(&args.as_str().into());
            }
        }
    }

    pub struct RustStdoutFns {
        pub stdout_fn: Option<Box<dyn Fn(String)>>,
        pub stderr_fn: Option<Box<dyn Fn(String)>>,
    }

    impl RustStdoutFns {
        pub fn new() -> Self {
            Self {
                stdout_fn: None,
                stderr_fn: None,
            }
        }

        pub fn set_stdout_fn(&mut self, f: Box<dyn Fn(String)>) {
            self.stdout_fn = Some(f);
        }

        pub fn set_stderr_fn(&mut self, f: Box<dyn Fn(String)>) {
            self.stderr_fn = Some(f);
        }
    }

    impl fmt::Debug for RustStdoutFns {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(f, "RustStdoutFns")
        }
    }
}
