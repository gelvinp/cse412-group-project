extends Node

var error_scene := preload("res://assets/scenes/main_menu/Error.tscn")
var error: ErrorDisplay

onready var form := $DBInitForm
onready var camera := $ViewportContainer/Viewport/EarthCamera/Camera


func _on_DBInitForm_initialize(address, port, database, username, password):	
	form.disable()
	_connect([address, port, database, username, password])
	print("Thread started")


func _connect(data):
	if DbConnection.connection.db_connect(data[0], data[1], data[2], data[3], data[4]):
		_on_connect()
	else:
		_auth_error()


func _on_connect() -> void:
	hide_form()
	DbConnection.countries = DbConnection.connection.get_countries()


func hide_form():
	var tween = create_tween().set_trans(Tween.TRANS_CUBIC).set_ease(Tween.EASE_OUT)
	tween.tween_property(form, "modulate", Color(1, 1, 1, 0), 0.1)
	tween.tween_property(camera, "frustum_offset", Vector2.ZERO, 0.6)


func show_form():
	form.enable()
	var tween = create_tween().set_trans(Tween.TRANS_CUBIC).set_ease(Tween.EASE_OUT)
	tween.tween_property(form, "modulate", Color(1, 1, 1, 1), 0.1)
	tween.tween_property(camera, "frustum_offset", Vector2(-0.286, 0), 0.6)


func _auth_error():
	error = error_scene.instance()
	add_child(error)
	error.error_text = "Authentication Error"
	error.connect("acknowledged", self, "_on_error_acknowledged")


func _on_error_acknowledged():
	show_form()


func _on_completed():
	print("Completed")
