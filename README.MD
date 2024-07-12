# Overview

This project uses:

1. [gdext](https://github.com/godot-rust/gdext) for Godot Rust bindings
2. [RustPython]() to interpret the Python code

Much thanks to the contributors to both of those projects!

This project would NOT be possible without their amazing work!

# Setup

To setup this project, you must follow these steps:

1. [Install rust](https://www.rust-lang.org/tools/install).
2. Run `cargo build` in the `godot_python` directory.
3. Open the Godot project and run the `test_scene.tscn` scene.

# ToDo

This project is definitely not finished. Here are some things that need to be done:

1. Routing Python's STDOUT and STDERR to Godot
2. Implementing more robust loading of Python files, such as an entire file tree, instead of loading individual modules
3. Building out a way to install dependencies from within Godot (ideally using something like pip)
