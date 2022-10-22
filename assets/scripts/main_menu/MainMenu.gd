extends Node


var _db_init

var error_scene := preload("res://assets/scenes/main_menu/Error.tscn")
var error: ErrorDisplay

var progress_scene := preload("res://assets/scenes/main_menu/Progress.tscn")
var progress: ProgressDisplay

onready var form := $DBInitForm
onready var camera := $ViewportContainer/Viewport/EarthCamera/Camera


func _on_DBInitForm_initialize(address, port, database, username, password):
	_db_init = DatabaseInitializer.new()
	
	progress = progress_scene.instance()
	progress.db_init = _db_init
	progress.connect("completed", self, "_on_completed")
	progress.connect("error", self, "_on_error")
	add_child(progress)
	
	hide_form()
	
	_db_init.init_db()


func hide_form():
	var tween = create_tween().set_trans(Tween.TRANS_CUBIC).set_ease(Tween.EASE_OUT)
	tween.tween_property(form, "modulate", Color(1, 1, 1, 0), 0.1)
	tween.tween_property(camera, "frustum_offset", Vector2.ZERO, 0.6)


func show_form():
	var tween = create_tween().set_trans(Tween.TRANS_CUBIC).set_ease(Tween.EASE_OUT)
	tween.tween_property(form, "modulate", Color(1, 1, 1, 1), 0.1)
	tween.tween_property(camera, "frustum_offset", Vector2(-0.286, 0), 0.6)


func _exit_tree():
	_db_init.cancel()


func _on_error(message: String):
	progress.fade_out()
	
	printerr(message)
	error = error_scene.instance()
	add_child(error)
	error.error_text = message
	error.connect("acknowledged", self, "_on_error_acknowledged")


func _on_error_acknowledged():
	show_form()


func _on_completed():
	print("Completed")
	progress.fade_out()
