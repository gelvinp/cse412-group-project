extends Node


var _worker: Thread
var _db_init


onready var form := $DBInitForm


func _on_DBInitForm_initialize(address, port, database, username, password):
	_db_init = DatabaseInitializer.new()
	_db_init.connect("error", self, "_on_error")
	_db_init.connect("completed", self, "_on_completed")
	_worker = Thread.new()
	_worker.start(_db_init, "init_db")
	
	var tween = create_tween()
	tween.tween_property(form, "modulate", Color(1, 1, 1, 0), 0.1)


func _exit_tree():
	if is_instance_valid(_worker) and _worker.is_active():
		_db_init.cancel()
		_worker.wait_to_finish()


func _on_error(message: String):
	printerr(message)


func _on_completed():
	print("Completed")
