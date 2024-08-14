extends Node2D

@export var python_vm_wrapper: PackedScene

@onready var result: Label = $UI/Control/Col/Row/Result
@onready var player: CharacterBody2D = $CharacterBody2D
@onready var code: CodeEdit = $UI/Control/Col/PythonEdit
@onready var move_label: Label = $UI/Control/Col/Row/Label
@onready var output_label: RichTextLabel = $UI/Control/Col/Margin/Output

var input: String = ""

var user_typing = false

const module_name = "char_module"
const MOVEMENT_SPEED = 300

# Called when the node enters the scene tree for the first time.
func _ready():
	PythonVM.stdout_updated.connect(_on_output)
	if PythonVM.is_ready:
		_on_python_vm_ready()
	else:
		PythonVM.python_vm_ready.connect(_on_python_vm_ready, ConnectFlags.CONNECT_ONE_SHOT)


func _on_python_vm_ready():
	print("Python VM Ready")
	PythonVM.load_module(module_name, code.text)

	var arg_test = PythonVM.call_python_function(module_name, "args_test", ["test"])
	print(arg_test)
	var kwarg_test = PythonVM.call_python_function(module_name, "kwargs_test", [], {"test": {"x": 1, "y": 2}})
	print(kwarg_test)
	var i_kwarg_test = PythonVM.call_python_function(module_name, "i_kwargs_test", [1], {"test": {"x": 1, "y": 2}})
	print(i_kwarg_test)


func _on_output(args) -> void:
	output_label.append_text(args)


func update_move_label() -> void:
	if user_typing:
		move_label.text = "Typing..."
	else:
		move_label.text = "Value: " + input


func get_keys(keys: String) -> Array:
	var key_list = keys.split("+")
	var key = key_list[key_list.size() - 1]

	var modifiers = {}
	# Get the remaining elements of the key list as modifiers
	for i in range(0, key_list.size() - 1):
		modifiers[key_list[i].to_lower()] = true

	return [key, modifiers]


func _input(event: InputEvent) -> void:
	if not PythonVM.is_ready:
		return

	if event is InputEventKey and not user_typing:
		if event.is_pressed():
			var i = get_keys(event.as_text_key_label())
			input = PythonVM.call_python_function(module_name, "input", [i[0], true], {})
		else: 
			var i = get_keys(event.as_text_key_label())
			input = PythonVM.call_python_function(module_name, "input", [i[0], false], {})
		
		update_move_label()


func _process(_delta: float) -> void:

	if not player:
		return
	
	if input == "up":
		player.velocity = Vector2(0, -1) * MOVEMENT_SPEED
	elif input == "down":
		player.velocity = Vector2(0, 1) * MOVEMENT_SPEED
	elif input == "left":
		player.velocity = Vector2(-1, 0) * MOVEMENT_SPEED
	elif input == "right":
		player.velocity = Vector2(1, 0) * MOVEMENT_SPEED
	else:
		player.velocity = Vector2(0, 0)
	player.move_and_slide()


func _on_deploy_pressed() -> void:
	PythonVM.load_module(module_name, code.text)


func _on_python_edit_focus_exited() -> void:
	user_typing = false
	update_move_label()

func _on_python_edit_focus_entered() -> void:
	user_typing = true
	update_move_label()
