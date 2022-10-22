class_name ProgressDisplay
extends MarginContainer


signal completed
signal error(message)


onready var _status_text := $MarginContainer/VBoxContainer/Status
onready var _progress := $MarginContainer/VBoxContainer/MarginContainer/Progress

var db_init: DatabaseInitializer


func _enter_tree():
	modulate = Color(1, 1, 1, 0)
	var tween = create_tween()
	tween.tween_property(self, "modulate", Color(1, 1, 1, 1), 0.1)


func _process(_delta):
	var status = db_init.get_status()
	
	if status["completed"]:
		emit_signal("completed")
		set_process(false)
		return
	
	if status.has("error"):
		set_process(false)
		emit_signal("error", status["error"])
		return
	
	_status_text.text = status["stage"]
	
	if status["discrete"]:
		_progress.percent_visible = true
		_progress.value = status["progress"]
		_progress.max_value = status["total"]
	else:
		_progress.percent_visible = false
		_progress.value = 0


func fade_out():
	var tween = create_tween()
	tween.tween_property(self, "modulate", Color(1, 1, 1, 0), 0.1)
	tween.tween_callback(self, "_on_tween_callback")


func _on_tween_callback():
	queue_free()
