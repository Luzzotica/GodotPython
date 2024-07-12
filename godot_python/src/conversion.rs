use godot::prelude::*;
use rustpython_vm::{PyObjectRef, VirtualMachine};

pub fn py_to_godot(vm: &VirtualMachine, py_obj: PyObjectRef) -> Variant {
    if let Ok(int_obj)
    if let Ok(int_obj) = i64::try_from_object(vm, py_obj.clone()) {
        Variant::from(int_obj)
    } else if let Ok(float_obj) = f64::try_from_object(vm, py_obj.clone()) {
        Variant::from(float_obj)
    } else if let Ok(str_obj) = String::try_from_object(vm, py_obj.clone()) {
        Variant::from(str_obj)
    } else if let Ok(list_obj) = Vec::<PyObjectRef>::try_from_object(vm, py_obj.clone()) {
        let mut gd_list = VariantArray::new();
        for item in list_obj {
            gd_list.push(py_to_godot(vm, item));
        }
        Variant::from(gd_list)
    } else {
        Variant::nil() // Return Nil if the type is not supported
    }
}
