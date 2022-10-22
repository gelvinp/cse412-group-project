class_name ErrorDisplay
extends MarginContainer

signal acknowledged


onready var _error_text := $MarginContainer/VBoxContainer/MarginContainer/ErrorText
var error_text setget _set_error, _get_error


func _enter_tree():
	modulate = Color(1, 1, 1, 0)
	var tween = create_tween()
	tween.tween_property(self, "modulate", Color(1, 1, 1, 1), 0.1)


func _set_error(error):
	_error_text.text = error


func _get_error():
	return _error_text.text


func _on_OKButton_pressed():
	emit_signal("acknowledged")
	var tween = create_tween()
	tween.tween_property(self, "modulate", Color(1, 1, 1, 0), 0.1)
	tween.tween_callback(self, "_on_tween_callback")


func _on_tween_callback():
	queue_free()
