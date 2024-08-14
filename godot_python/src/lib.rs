#[cfg(not(target_arch = "wasm32"))]
use godot::prelude::*;

// mod module_loader;
pub mod python_vm_common;
#[cfg(not(target_arch = "wasm32"))]
mod python_vm_godot;
#[cfg(target_arch = "wasm32")]
mod python_vm_web;

#[cfg(not(target_arch = "wasm32"))]
struct GodotPython;

#[cfg(not(target_arch = "wasm32"))]
#[gdextension]
unsafe impl ExtensionLibrary for GodotPython {}
