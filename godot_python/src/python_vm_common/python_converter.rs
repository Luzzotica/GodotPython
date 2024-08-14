use rustpython_vm::{builtins::PyBaseException, PyRef, TryFromObject, VirtualMachine};

pub fn unwrap_error(vm: &VirtualMachine, error: PyRef<PyBaseException>) -> String {
    let mut s = String::new();
    error.args().into_iter().for_each({
        |v| {
            if let Ok(str_obj) = String::try_from_object(vm, v.clone()) {
                s.push_str(&format!("{:?}\n", str_obj));
            } else {
                s.push_str(&format!("{:?}\n", v));
            }
        }
    });
    s
}
