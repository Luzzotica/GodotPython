# Overview

This project uses:

1. [gdext](https://github.com/godot-rust/gdext) for Godot Rust bindings
2. [RustPython](https://github.com/RustPython/RustPython) to interpret the Python code

Much thanks to the contributors to both of those projects!

This project would NOT be possible without their amazing work!

# Project Goal

The goal of this project is to allow for Python scripting in Godot, without the need for GDNative or any other external dependencies.

This allows for Python programming inside of Godot, which can enable rapid prototyping, or building games that require (or desire) reprogrammable components.

# Setup

## Desktop

To setup this project, you must follow these steps:

1. [Install rust](https://www.rust-lang.org/tools/install).
2. Run `cargo build` in the `godot_python` directory.
3. Open the Godot project and run the `test_scene.tscn` scene.

## Web

To run the project in a browser, it should just work, as the WASM package is downloaded from the cdn directly (and doesn't require any additional building on your part).

# ToDo

This project is definitely not finished. Here are some things that need to be done:

1. Implementing more robust loading of Python files, such as an entire file tree, instead of loading individual modules
2. Building out a way to install dependencies from within Godot (ideally using something like pip)
