use godot::engine::NodeVirtual;
use godot::prelude::*;
// use rustpython;
use rustpython_vm as vm;

#[derive(GodotClass)]
#[class(base=Node)]
struct MarkdownParser {
    #[base]
    node: Base<Node>,
}

#[godot_api]
impl NodeVirtual for MarkdownParser {
    fn init(node: Base<Node>) -> Self {
        Self { node }
    }
}

#[godot_api]
impl MarkdownParser {
    #[func]
    fn markdown_to_html(&mut self, markdown: String) {
        godot_print!("Running Python code: {}", code);
    }
}
