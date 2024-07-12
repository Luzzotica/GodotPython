extends Node2D

@onready var python_runner = $PythonRunner

@onready var result: Label = $UI/Control/Col/Row/Result
@onready var character: CharacterBody2D = $CharacterBody2D
@onready var code: CodeEdit = $UI/Control/Col/CodeEdit
@onready var move_label: Label = $UI/Control/Col/Row/Label

var input: String = ""

const module_name = "char_module"
const MOVEMENT_SPEED = 100

# Called when the node enters the scene tree for the first time.
func _ready():
	print(code)
	deploy_code(code.text)
	# var python_expressions = ["-200", "'test'", "-200.1"]
	# for expr in python_expressions:
	# 	print("Expression Result: ", python_runner.run_python(expr))

	

	# var i = python_runner.input("test_module", "swag")
	# print("input: ", i)


func deploy_code(code: String) -> void:
	var loaded_module = python_runner.load_module(module_name, code)
	print("loaded module: ", loaded_module)
	result.text = loaded_module


func update_move_label() -> void:
	move_label.text = "Value: " + input


func _input(event: InputEvent) -> void:
	if event is InputEventKey:
		if event.is_pressed():
			var key = event.as_text_key_label()
			print("key down: ", key)
			input = python_runner.input_down(module_name, key)
		else: 
			var key = event.as_text_key_label()
			print("key up: ", key)
			if input == python_runner.input_up(module_name, key):
				input = "Idling"
		
		update_move_label()


func _process(_delta: float) -> void:
	if not character:
		return
	
	if input == "up":
		character.velocity = Vector2(0, -1) * MOVEMENT_SPEED
	elif input == "down":
		character.velocity = Vector2(0, 1) * MOVEMENT_SPEED
	elif input == "left":
		character.velocity = Vector2(-1, 0) * MOVEMENT_SPEED
	elif input == "right":
		character.velocity = Vector2(1, 0) * MOVEMENT_SPEED
	else:
		character.velocity = Vector2(0, 0)
	character.move_and_slide()


func _on_deploy_pressed() -> void:
	deploy_code(code.text)