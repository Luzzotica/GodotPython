use indexmap::IndexMap;
use js_sys::{Array, ArrayBuffer, BigInt, Object, Reflect, Uint8Array};
use rustpython_vm::{
    builtins::{PyBaseExceptionRef, PyDict, PyList},
    function::{FuncArgs, KwArgs},
    PyObjectRef, PyResult, TryFromObject, VirtualMachine,
};
use wasm_bindgen::{prelude::*, JsCast};

use crate::python_vm_common::CommonPythonVM;

pub fn convert_py_to_js_common(common_vm: &CommonPythonVM, value: PyObjectRef) -> JsValue {
    common_vm
        .interpreter
        .enter(|vm| convert_py_to_js(vm, value))
}

pub fn convert_py_to_js(vm: &VirtualMachine, value: PyObjectRef) -> JsValue {
    if let Ok(int_obj) = i64::try_from_object(&vm, value.clone()) {
        JsValue::from_f64(int_obj as f64)
    } else if let Ok(float_obj) = f64::try_from_object(&vm, value.clone()) {
        JsValue::from_f64(float_obj)
    } else if let Ok(str_obj) = String::try_from_object(&vm, value.clone()) {
        JsValue::from_str(&str_obj)
    } else if let Ok(bool_obj) = bool::try_from_object(&vm, value.clone()) {
        JsValue::from_bool(bool_obj)
    } else if let Some(list) = value.downcast_ref::<PyList>() {
        let js_arr = Array::new();
        list.borrow_vec().iter().for_each(|item| {
            js_arr.push(&convert_py_to_js(vm, item.clone()));
        });
        JsValue::from(js_arr)
        // for item in list.elements() {
        //     js_arr.push(&convery_py_to_js(vm, item.clone()));
        // }
    } else if let Some(dict) = value.downcast_ref::<PyDict>() {
        let js_obj = Object::new();
        dict.into_iter().for_each(|(key, val)| {
            Reflect::set(
                &js_obj,
                &convert_py_to_js(vm, key.clone()),
                &convert_py_to_js(vm, val.clone()),
            )
            .expect("property to be settable");
        });
        JsValue::from(js_obj)
    } else {
        JsValue::UNDEFINED
    }
}

pub fn convert_js_to_py(vm: &VirtualMachine, js_val: JsValue) -> PyObjectRef {
    if js_val.is_bigint() {
        let bi = BigInt::from(js_val);
        let bis = bi.to_string(10).unwrap().as_string().unwrap();

        if let Ok(bigint) = bis.parse::<i64>() {
            vm.ctx.new_int(bigint).into()
        } else {
            vm.ctx.new_int(0).into()
        }
    } else if let Some(fl) = js_val.as_f64() {
        vm.ctx.new_float(fl).into()
    } else if let Some(b) = js_val.as_bool() {
        vm.ctx.new_bool(b).into()
    } else if let Some(s) = js_val.as_string() {
        vm.ctx.new_str(s.as_str()).into()
    } else if Array::is_type_of(&js_val) {
        let js_arr: Array = js_val.into();
        let elems = js_arr
            .values()
            .into_iter()
            .map(|val| convert_js_to_py(vm, val.expect("Iteration over array failed")))
            .collect();
        vm.ctx.new_list(elems).into()
    } else if ArrayBuffer::is_type_of(&js_val) {
        // unchecked_ref because if it's not an ArrayBuffer it could either be a TypedArray
        // or a DataView, but they all have a `buffer` property
        let u8_array = js_sys::Uint8Array::new(
            &js_val
                .dyn_ref::<ArrayBuffer>()
                .cloned()
                .unwrap_or_else(|| js_val.unchecked_ref::<Uint8Array>().buffer()),
        );
        let mut vec = vec![0; u8_array.length() as usize];
        u8_array.copy_to(&mut vec);
        vm.ctx.new_bytes(vec).into()
    } else if Object::is_type_of(&js_val) {
        let dict = vm.ctx.new_dict();
        for pair in object_entries(&Object::from(js_val)) {
            let (key, val) = pair.expect("iteration over object to not fail");
            let py_val = convert_js_to_py(vm, val);
            dict.set_item(
                String::from(js_sys::JsString::from(key)).as_str(),
                py_val,
                vm,
            )
            .unwrap();
        }
        dict.into()
    } else if js_val.is_function() {
        let func = js_sys::Function::from(js_val);
        vm.new_function(
            vm.ctx.intern_str(String::from(func.name())).as_str(),
            move |args: FuncArgs, vm: &VirtualMachine| -> PyResult {
                let this = Object::new();
                for (k, v) in args.kwargs {
                    Reflect::set(&this, &k.into(), &convert_py_to_js(vm, v))
                        .expect("property to be settable");
                }
                let js_args = args
                    .args
                    .into_iter()
                    .map(|v| convert_py_to_js(vm, v))
                    .collect::<Array>();
                func.apply(&this, &js_args)
                    .map(|val| convert_js_to_py(vm, val))
                    .map_err(|err| js_err_to_py_err(vm, &err))
            },
        )
        .into()
    } else if let Some(err) = js_val.dyn_ref::<js_sys::Error>() {
        js_err_to_py_err(vm, err).into()
    } else if js_val.is_undefined() {
        // Because `JSON.stringify(undefined)` returns undefined
        vm.ctx.none()
    } else {
        vm.ctx.none()
    }
}

pub fn js_err_to_py_err(vm: &VirtualMachine, js_err: &JsValue) -> PyBaseExceptionRef {
    match js_err.dyn_ref::<js_sys::Error>() {
        Some(err) => {
            let exc_type = match String::from(err.name()).as_str() {
                "TypeError" => vm.ctx.exceptions.type_error,
                "ReferenceError" => vm.ctx.exceptions.name_error,
                "SyntaxError" => vm.ctx.exceptions.syntax_error,
                _ => vm.ctx.exceptions.exception_type,
            }
            .to_owned();
            vm.new_exception_msg(exc_type, err.message().into())
        }
        None => vm.new_exception_msg(
            vm.ctx.exceptions.exception_type.to_owned(),
            format!("{js_err:?}"),
        ),
    }
}

pub fn convert_js_arr_to_args(common_vm: &CommonPythonVM, arr: Array) -> Vec<PyObjectRef> {
    common_vm.interpreter.enter(|vm| {
        arr.into_iter()
            .map(|val| convert_js_to_py(vm, val))
            .collect()
    })
}

pub fn convert_js_obj_to_kwargs(common_vm: &CommonPythonVM, obj: Object) -> KwArgs {
    common_vm.interpreter.enter(|vm| {
        let mut map = IndexMap::new();

        object_entries(&obj).for_each(|pair| {
            let (key, val) = pair.expect("iteration over object to not fail");
            map.insert(
                String::from(js_sys::JsString::from(key)),
                convert_js_to_py(vm, val),
            );
        });
        KwArgs::new(map)
    })
}

pub fn object_entries(obj: &Object) -> impl Iterator<Item = Result<(JsValue, JsValue), JsValue>> {
    Object::entries(obj).values().into_iter().map(|pair| {
        pair.map(|pair| {
            let key = Reflect::get(&pair, &"0".into()).unwrap();
            let val = Reflect::get(&pair, &"1".into()).unwrap();
            (key, val)
        })
    })
}

#[cfg(test)]
pub mod tests {

    use js_sys::Map;
    use rustpython_vm::{
        builtins::{PyFloat, PyInt, PyStr},
        convert::ToPyObject,
        PyPayload,
    };
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    #[wasm_bindgen_test]
    fn test_convert_py_to_js_int() {
        let common_vm = CommonPythonVM::init();
        let result = common_vm.interpreter.enter(|vm| {
            let value = PyInt::from(42).into_pyobject(vm);
            convert_py_to_js(&vm, value)
        });
        assert_eq!(result, JsValue::from_f64(42.0));
    }

    #[wasm_bindgen_test]
    fn test_convert_py_to_js_float() {
        let common_vm = CommonPythonVM::init();
        let result = common_vm.interpreter.enter(|vm| {
            let value = PyFloat::from(3.14).into_pyobject(vm);
            convert_py_to_js(&vm, value)
        });
        assert_eq!(result, JsValue::from_f64(3.14));
    }

    #[wasm_bindgen_test]
    fn test_convert_py_to_js_str() {
        let common_vm = CommonPythonVM::init();
        let result = common_vm.interpreter.enter(|vm| {
            let value = PyStr::from("hello").into_pyobject(vm);
            convert_py_to_js(&vm, value)
        });
        assert_eq!(result, JsValue::from_str("hello"));
    }

    // #[wasm_bindgen_test]
    // fn test_convert_py_to_js_bool() {
    //     let common_vm = CommonPythonVM::init();
    //     let result = common_vm.interpreter.enter(|vm| {
    //         let value = PyBool::from(1).into_pyobject(vm);
    //         convert_py_to_js(&vm, value)
    //     });
    //     assert_eq!(result, JsValue::from_bool(true));
    // }

    // #[wasm_bindgen_test]
    // fn test_convert_py_to_js_list() {
    //     let common_vm = CommonPythonVM::init();
    //     common_vm.interpreter.enter(|vm| {
    //         let value = PyListRef::from(vec![
    //             PyInt::from(1).into_pyobject(vm),
    //             PyInt::from(2).into_pyobject(vm),
    //             PyInt::from(3).into_pyobject(vm),
    //         ])
    //         .into_pyobject(vm);
    //         let result = convert_py_to_js(&vm, value);
    //         let expected = JsValue::from(vec![
    //             JsValue::from_f64(1.0),
    //             JsValue::from_f64(2.0),
    //             JsValue::from_f64(3.0),
    //         ]);
    //         assert_eq!(result, expected);
    //     });
    // }

    // #[wasm_bindgen_test]
    // fn test_convert_py_to_js_dict() {
    //     let common_vm = CommonPythonVM::init();
    //     common_vm.interpreter.enter(|vm| {
    //         let dict = vm.ctx.new_dict();
    //         dict.set_item("key1", PyInt::new(1).into_pyobject(vm), vm)
    //             .unwrap();
    //         dict.set_item("key2", PyInt::new(2).into_pyobject(vm), vm)
    //             .unwrap();
    //         let value = dict.into_pyobject(vm);
    //         let result = convert_py_to_js(&vm, value);
    //         let expected = JsValue::from(Object::from_iter(vec![
    //             ("key1".into(), JsValue::from_f64(1.0)),
    //             ("key2".into(), JsValue::from_f64(2.0)),
    //         ]));
    //         assert_eq!(result, expected);
    //     });
    // }

    #[wasm_bindgen_test]
    fn test_convert_py_to_js_undefined() {
        let common_vm = CommonPythonVM::init();
        common_vm.interpreter.enter(|vm| {
            let value = PyObjectRef::from(vm.ctx.none());
            let result = convert_py_to_js(&vm, value);
            assert_eq!(result, JsValue::UNDEFINED);
        });
    }

    #[wasm_bindgen_test]
    fn test_convert_js_to_py() {
        let common_vm = CommonPythonVM::init();
        let (
            int_r,
            int_r2,
            fl_r,
            bool_r,
            str_r,
            list_r,
            dict_r,
            int_e,
            int_e2,
            fl_e,
            bool_e,
            str_e,
            list_e,
            dict_e,
        ) = common_vm.interpreter.enter(|vm| {
            let int_r = convert_js_to_py(&vm, BigInt::from(42).into());
            let int_r2 = convert_js_to_py(&vm, BigInt::from(-42).into());
            let fl_r = convert_js_to_py(&vm, JsValue::from_f64(42.5));
            let bool_r = convert_js_to_py(&vm, JsValue::from_bool(true));
            let str_r = convert_js_to_py(&vm, JsValue::from_str("test"));
            let arr_r = convert_js_to_py(
                &vm,
                JsValue::from(Array::of2(
                    &JsValue::from_f64(42.0),
                    &JsValue::from_bool(true),
                )),
            );
            // Create a new JavaScript object
            let obj = Object::new();
            Reflect::set(&obj, &JsValue::from("key1"), &JsValue::from("value1")).unwrap();
            Reflect::set(&obj, &JsValue::from("key2"), &BigInt::from(42).into()).unwrap();
            Reflect::set(&obj, &JsValue::from("key3"), &JsValue::from(42.5)).unwrap();
            Reflect::set(&obj, &JsValue::from("key4"), &JsValue::from(true)).unwrap();
            let obj2 = Object::new();
            Reflect::set(&obj2, &JsValue::from("key1"), &JsValue::from("value1")).unwrap();
            Reflect::set(&obj2, &JsValue::from("key2"), &JsValue::from(1.5)).unwrap();
            Reflect::set(&obj, &JsValue::from("key5"), &obj2).unwrap();
            let dict_r = convert_js_to_py(&vm, JsValue::from(obj));

            let dict_p = vm.ctx.new_dict();

            dict_p
                .set_item("key1", vm.ctx.new_str("value1").to_pyobject(vm), vm)
                .unwrap();
            dict_p
                .set_item("key2", vm.ctx.new_int(42).to_pyobject(vm), vm)
                .unwrap();
            dict_p
                .set_item("key3", vm.ctx.new_float(42.5).to_pyobject(vm), vm)
                .unwrap();
            dict_p
                .set_item("key4", vm.ctx.new_bool(true).to_pyobject(vm), vm)
                .unwrap();
            let dict_p2 = vm.ctx.new_dict();
            dict_p2
                .set_item("key1", vm.ctx.new_str("value1").to_pyobject(vm), vm)
                .unwrap();
            dict_p2
                .set_item("key2", vm.ctx.new_float(1.5).to_pyobject(vm), vm)
                .unwrap();
            dict_p
                .set_item("key5", dict_p2.to_pyobject(vm), vm)
                .unwrap();

            (
                int_r,
                int_r2,
                fl_r,
                bool_r,
                str_r,
                arr_r,
                dict_r,
                vm.ctx.new_int(42).to_pyobject(vm),
                vm.ctx.new_int(-42).to_pyobject(vm),
                vm.ctx.new_float(42.5).to_pyobject(vm),
                vm.ctx.new_bool(true).to_pyobject(vm),
                vm.ctx.new_str("test").to_pyobject(vm),
                vm.ctx
                    .new_list(vec![
                        vm.ctx.new_float(42.0).to_pyobject(vm),
                        vm.ctx.new_bool(true).to_pyobject(vm),
                    ])
                    .to_pyobject(vm),
                dict_p.to_pyobject(vm),
            )
        });

        assert_eq!(format!("{:?}", int_r), format!("{:?}", int_e));
        assert_eq!(format!("{:?}", int_r2), format!("{:?}", int_e2));
        assert_eq!(format!("{:?}", fl_r), format!("{:?}", fl_e));
        assert_eq!(format!("{:?}", bool_r), format!("{:?}", bool_e));
        assert_eq!(format!("{:?}", str_r), format!("{:?}", str_e));
        // Iterate through list and compare each value of list and dict
        let py_list_r = list_r.downcast_ref::<PyList>().unwrap();
        let py_list_e = list_e.downcast_ref::<PyList>().unwrap();
        py_list_e
            .borrow_vec()
            .iter()
            .zip(py_list_r.borrow_vec().iter())
            .for_each(|(item_r, item_e)| {
                assert_eq!(format!("{:?}", item_r), format!("{:?}", item_e));
            });

        let py_dict_r = dict_r.downcast::<PyDict>().unwrap();
        let py_dict_e = dict_e.downcast::<PyDict>().unwrap();
        py_dict_r.into_iter().zip(py_dict_e.into_iter()).for_each(
            |((key1, val1), (key2, val2))| {
                let val1_dict = val1.clone().downcast::<PyDict>();
                match val1_dict {
                    Ok(val1_dict) => {
                        let val2_dict = val2.downcast::<PyDict>().unwrap();
                        val1_dict.into_iter().zip(val2_dict.into_iter()).for_each(
                            |((key1, val1), (key2, val2))| {
                                assert_eq!(format!("{:?}", key1), format!("{:?}", key2));
                                assert_eq!(format!("{:?}", val1), format!("{:?}", val2));
                            },
                        );
                    }
                    Err(_) => {
                        assert_eq!(format!("{:?}", key1), format!("{:?}", key2));
                        assert_eq!(format!("{:?}", val1), format!("{:?}", val2));
                    }
                }
            },
        );
    }

    #[wasm_bindgen_test]
    fn test_convert_js_arr_to_args() {
        let common_vm = CommonPythonVM::init();
        let (result, expected) = common_vm.interpreter.enter(|vm| {
            let value = Array::new();
            value.push(&JsValue::from_f64(42.0));
            value.push(&JsValue::from_bool(true));
            value.push(&JsValue::from_str("test"));

            let result = convert_js_arr_to_args(&common_vm, value);

            let expected = vec![
                vm.ctx.new_float(42.0).to_pyobject(vm),
                vm.ctx.new_bool(true).to_pyobject(vm),
                vm.ctx.new_str("test").to_pyobject(vm),
            ];

            assert_eq!(result.len(), expected.len());
            (result, expected)
        });

        for (v1, v2) in result.iter().zip(expected.iter()) {
            assert_eq!(format!("{:?}", v1), format!("{:?}", v2));
        }
    }

    #[wasm_bindgen_test]
    fn test_convert_empty_js_arr_to_args() {
        let common_vm = CommonPythonVM::init();
        let value = Array::new();

        let result = convert_js_arr_to_args(&common_vm, value);

        let expected: Vec<PyObjectRef> = vec![];

        assert_eq!(result.len(), expected.len());

        for (v1, v2) in result.iter().zip(expected.iter()) {
            assert_eq!(format!("{:?}", v1), format!("{:?}", v2));
        }
    }

    #[wasm_bindgen_test]
    fn test_convert_js_obj_to_kwargs() {
        let common_vm = CommonPythonVM::init();
        let entries = Map::new();
        entries.set(&"bool".into(), &JsValue::from_bool(true));
        entries.set(&"str".into(), &JsValue::from_str("test"));
        entries.set(&"num".into(), &JsValue::from_f64(42.0));
        let value = Object::from_entries(&entries).unwrap();

        let result = convert_js_obj_to_kwargs(&common_vm, value);
        let expected = common_vm.interpreter.enter(|vm| {
            let mut map = IndexMap::new();
            map.insert("bool".to_string(), vm.ctx.new_bool(true).to_pyobject(vm));
            map.insert("str".to_string(), vm.ctx.new_str("test").to_pyobject(vm));
            map.insert("num".to_string(), vm.ctx.new_float(42.0).to_pyobject(vm));
            map
        });

        for (key, val) in result.into_iter() {
            let expected_val = expected.get(&key).unwrap();
            // assert_eq!(format!("{:?}", key), format!("{:?}", expected_key));
            assert_eq!(format!("{:?}", val), format!("{:?}", expected_val));
        }
    }
}
