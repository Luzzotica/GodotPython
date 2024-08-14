use godot::prelude::*;
use indexmap::IndexMap;
use rustpython_vm::{
    builtins::{PyDict, PyList},
    function::KwArgs,
    PyObjectRef, TryFromObject, VirtualMachine,
};

use crate::python_vm_common::CommonPythonVM;

pub fn convert_py_to_variant_common(common_vm: &CommonPythonVM, value: PyObjectRef) -> Variant {
    common_vm
        .interpreter
        .enter(|vm| convert_py_object_to_variant(vm, value))
}

// pub fn convert_variant_to_py_common(common_vm: &CommonPythonVM, value: Variant) -> PyObjectRef {
//     common_vm
//         .interpreter
//         .enter(|vm| convert_variant_to_py_object(vm, value))
// }

pub fn convert_py_object_to_variant(vm: &VirtualMachine, value: PyObjectRef) -> Variant {
    // Handle success case
    // godot_print!("Success: {:?}", value);
    if let Ok(int_obj) = i64::try_from_object(&vm, value.clone()) {
        Variant::from(int_obj)
    } else if let Ok(float_obj) = f64::try_from_object(&vm, value.clone()) {
        Variant::from(float_obj)
    } else if let Ok(str_obj) = String::try_from_object(&vm, value.clone()) {
        Variant::from(str_obj)
    } else if let Ok(bool_obj) = bool::try_from_object(&vm, value.clone()) {
        Variant::from(bool_obj)
    } else if let Some(list) = value.downcast_ref::<PyList>() {
        let mut arr = VariantArray::new();
        list.borrow_vec().iter().for_each(|item| {
            arr.push(convert_py_object_to_variant(vm, item.clone()));
        });
        Variant::from(arr)
        // for item in list.elements() {
        //     js_arr.push(&convery_py_to_js(vm, item.clone()));
        // }
    } else if let Some(dict) = value.downcast_ref::<PyDict>() {
        let mut obj = Dictionary::new();
        dict.into_iter().for_each(|(key, val)| {
            obj.insert(
                convert_py_object_to_variant(vm, key.clone()),
                convert_py_object_to_variant(vm, val.clone()),
            )
            .expect("property to be settable");
        });
        Variant::from(obj)
    } else {
        Variant::nil()
    }

    // let arr = VariantArray::new();
}

pub fn convert_variant_to_py_object(virt: &VirtualMachine, value: Variant) -> PyObjectRef {
    match value.get_type() {
        VariantType::NIL => virt.ctx.none(),
        VariantType::BOOL => virt.ctx.new_bool(bool::from_variant(&value)).into(),
        VariantType::INT => virt.ctx.new_int(i32::from_variant(&value)).into(),
        VariantType::FLOAT => virt.ctx.new_float(f64::from_variant(&value)).into(),
        VariantType::STRING => virt.ctx.new_str(String::from_variant(&value)).into(),
        VariantType::ARRAY => {
            let arr = VariantArray::from_variant(&value);
            let mut elements = Vec::new();
            for i in 0..arr.len() {
                elements.push(convert_variant_to_py_object(virt, arr.get(i).unwrap()));
            }
            let list = virt.ctx.new_list(elements);
            list.into()
        }
        VariantType::DICTIONARY => {
            let dict = Dictionary::from_variant(&value);
            let keys = dict.keys_array();
            let py_dict = virt.ctx.new_dict();
            for i in 0..keys.len() {
                let v_key = keys.get(i).unwrap();
                let key = String::from_variant(&v_key);
                let _ = py_dict.set_item(
                    &key.clone(),
                    convert_variant_to_py_object(virt, dict.get(key).unwrap()),
                    virt,
                );
                // elements.push((convert_variant_to_py_object(virt, value),));
            }
            py_dict.into()
        }
        _ => virt.ctx.none(),
    }
}

pub fn convert_variant_arr_to_args(
    common_vm: &CommonPythonVM,
    arr: VariantArray,
) -> Vec<PyObjectRef> {
    common_vm.interpreter.enter(|vm| {
        let mut elements = Vec::new();
        for i in 0..arr.len() {
            elements.push(convert_variant_to_py_object(vm, arr.get(i).unwrap()));
        }
        elements
    })
}

pub fn convert_variant_dict_to_kwargs(common_vm: &CommonPythonVM, dict: Dictionary) -> KwArgs {
    common_vm.interpreter.enter(|vm| {
        let keys = dict.keys_array();
        let mut map = IndexMap::new();
        for i in 0..keys.len() {
            let v_key = keys.get(i).unwrap();
            let key = String::from_variant(&v_key);
            let _ = map.insert(
                key.clone(),
                convert_variant_to_py_object(vm, dict.get(key).unwrap()),
            );
        }
        KwArgs::new(map)
    })
}
