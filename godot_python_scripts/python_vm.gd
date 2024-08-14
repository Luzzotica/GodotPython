extends Node

@export var python_vm_wrapper: PackedScene

var python_vm
var is_ready = false
var output: String = ""

var is_web: bool = false
var _wasm_version = "0.7.3"
var _window
var _document
var _console
var _import_script_loaded_callback
var _output_callback

signal python_vm_ready()
signal stdout_updated(t: String)


func _ready():
	is_web = OS.get_name() == "Web"
	if is_web:
		_window = JavaScriptBridge.get_interface("window")
		_document = JavaScriptBridge.get_interface("document")
		_console = JavaScriptBridge.get_interface("console")
		_import_script_loaded_callback  = JavaScriptBridge.create_callback(_script_loaded)
		_output_callback = JavaScriptBridge.create_callback(append_output)
		import_wasm_python_vm()
	else:
		add_child(python_vm_wrapper.instantiate())
		python_vm = $GodotPythonVMWrapper.python_vm
		python_vm.setup_stdout(self)
		setup_vm()
	

func import_wasm_python_vm():
	# print("importing wasm python vm")

	JavaScriptBridge.eval("""
window.load_python_vm = async () => {
	console.log('init module');
	let { default: initWasm, WasmPythonVM } = await import('https://cdn.jsdelivr.net/npm/godot_python@%s/godot_python.js');
	// Initialize the WASM module
	console.log('init wasm');
	await initWasm("https://cdn.jsdelivr.net/npm/godot_python@%s/godot_python_bg.wasm");

	// Create an instance of WasmPythonVM
	console.log('create python vm');
	const python_vm = new WasmPythonVM();
	window.python_vm = python_vm;
	console.log("PythonVM: ", window.python_vm);

	// console.log("Calling script loaded");
	// window.script_loaded(true);
}
	""" % [_wasm_version, _wasm_version])

	_window.load_python_vm().then(_import_script_loaded_callback)


func _script_loaded(_args) -> void:
	# print("script loaded: ", args)
	python_vm = _window.python_vm
	python_vm.setup_stdout(_output_callback)

	setup_vm()


func setup_vm():
	for c in $Modules.get_children():
		load_module(c.module_name, c.module_code)

	is_ready = true
	python_vm_ready.emit()


func append_output(args) -> void:
	# print(args)
	# print("append_output: ", args)
	if args is String:
		output += args
		stdout_updated.emit(args)
		# print('append_output: ', args)
	else:
		for text in args:
			output += text
			stdout_updated.emit(text)
			# print('append_output: ', text)


func eval(expr: String):
	if not python_vm:
		return "Python VM not loaded"

	return python_vm.eval(expr)


func load_module(m_name: String, c: String) -> bool:
	if not python_vm:
		return false
	
	var loaded_module = python_vm.load_module(m_name, c)
	# print("loaded module: ", loaded_module)
	# result.text = loaded_module

	if loaded_module == "Success":
		append_output("[color=green]Loaded Module: %s[/color]\n" % loaded_module)
		return true
	else:
		append_output("[color=red]%s[/color]\n" % loaded_module)
		return false



func call_python_function(module_name: String, function_name: String, args: Array = [], kwargs: Dictionary = {}) -> Variant:
	if not python_vm:
		return "Python VM not loaded"

	var args_actual = create_args(args)
	var kwargs_actual = create_kwargs(kwargs)
	var result = python_vm.call_python_function(module_name, function_name, args_actual, kwargs_actual)

	if is_web and result is JavaScriptObject:
		_console.log(result)
		var arr = []
		for i in range(0, result.length):
			arr.append(result[i])
		return arr

	return result


func create_args(args: Array):
	if is_web:
		# Create a javascript array with the values
		# print("Args: ", args, " Size: ", args.size())
		# JavaScriptBridge.eval("let arr = new Array(0); console.log(arr);")
		var arr = JavaScriptBridge.create_object("Array")
		for i in range(0, args.size()):
			if args[i] is Array:
				arr.push(create_args(args[i]))
			elif args[i] is Dictionary:
				arr.push(create_kwargs(args[i]))
			else:
				arr.push(args[i])
		# _console.log("create_args")
		# _console.log(args)
		# _console.log(arr)
		return arr
	else:
		return args


func create_kwargs(kwargs: Dictionary):
	if is_web:
		# Create a javascript object with the values
		var obj = JavaScriptBridge.create_object("Object")
		for key in kwargs.keys():
			if kwargs[key] is Array:
				obj[key] = create_args(kwargs[key])
			elif kwargs[key] is Dictionary:
				obj[key] = create_kwargs(kwargs[key])
			else:
				obj[key] = kwargs[key]
		# _console.log("create_kwargs")
		# _console.log(kwargs)
		# _console.log(obj)
		return obj
	else:
		return kwargs
