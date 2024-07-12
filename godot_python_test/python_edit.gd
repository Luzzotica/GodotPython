extends CodeEdit

@export var swag: String

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
  var keyword_color = Color(0.5, 0.4, 0.9)
  syntax_highlighter.keyword_colors = {
    "and": keyword_color,
    "as": keyword_color,
    "assert": keyword_color,
    "break": keyword_color,
    "class": keyword_color,
    "continue": keyword_color,
    "def": keyword_color,
    "del": keyword_color,
    "elif": keyword_color,
    "else": keyword_color,
    "except": keyword_color,
    "finally": keyword_color,
    "for": keyword_color,
    "from": keyword_color,
    "global": keyword_color,
    "if": keyword_color,
    "import": keyword_color,
    "in": keyword_color,
    "is": keyword_color,
    "lambda": keyword_color,
    "nonlocal": keyword_color,
    "not": keyword_color,
    "or": keyword_color,
    "pass": keyword_color,
    "raise": keyword_color,
    "return": keyword_color,
    "try": keyword_color,
    "while": keyword_color,
    "with": keyword_color,
    "yield": keyword_color,
  }
