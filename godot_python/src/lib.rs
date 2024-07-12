use godot::prelude::*;

mod python_runner;
// mod stdout_override;

struct GodotPython;

#[gdextension]
unsafe impl ExtensionLibrary for GodotPython {}
