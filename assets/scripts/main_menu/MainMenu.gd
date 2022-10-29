extends Node


var _db_init

var error_scene := preload("res://assets/scenes/main_menu/Error.tscn")
var error: ErrorDisplay

var progress_scene := preload("res://assets/scenes/main_menu/Progress.tscn")
var progress: ProgressDisplay

onready var form := $DBInitForm
onready var camera := $ViewportContainer/Viewport/EarthCamera/Camera

var thread: Thread


func _on_DBInitForm_initialize(address, port, database, username, password):	
	form.disable()
	if is_instance_valid(thread):
		thread.wait_to_finish()
	
	thread = Thread.new()
	thread.start(self, "_connect", [address, port, database, username, password])


func _connect(data):
	_db_init = DatabaseInitializer.new()
	if not _db_init.connect(data[0], data[1], data[2], data[3], data[4]):
		hide_form()
		error = error_scene.instance()
		add_child(error)
		error.error_text = "Unable to connect to database"
		error.connect("acknowledged", self, "_on_error_acknowledged")
		return
	
	hide_form()	
	progress = progress_scene.instance()
	progress.db_init = _db_init
	progress.connect("completed", self, "_on_completed")
	progress.connect("error", self, "_on_error")
	add_child(progress)
	_db_init.init_db()


func hide_form():
	var tween = create_tween().set_trans(Tween.TRANS_CUBIC).set_ease(Tween.EASE_OUT)
	tween.tween_property(form, "modulate", Color(1, 1, 1, 0), 0.1)
	tween.tween_property(camera, "frustum_offset", Vector2.ZERO, 0.6)


func show_form():
	form.enable()
	var tween = create_tween().set_trans(Tween.TRANS_CUBIC).set_ease(Tween.EASE_OUT)
	tween.tween_property(form, "modulate", Color(1, 1, 1, 1), 0.1)
	tween.tween_property(camera, "frustum_offset", Vector2(-0.286, 0), 0.6)


func _exit_tree():
	if is_instance_valid(_db_init):
		_db_init.cancel()
	
	if is_instance_valid(thread):
		thread.wait_to_finish()


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
