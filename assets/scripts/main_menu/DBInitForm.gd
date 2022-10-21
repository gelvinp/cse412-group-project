extends PanelContainer

signal initialize(address, port, database)


onready var address := $MarginContainer/VBoxContainer/VBoxContainer/HBoxContainer/Address
onready var port := $MarginContainer/VBoxContainer/VBoxContainer/HBoxContainer2/Port
onready var database := $MarginContainer/VBoxContainer/VBoxContainer/HBoxContainer3/DatabaseName
onready var initalize := $MarginContainer/VBoxContainer/HBoxContainer2/Initialize


func _ready():
	modulate = Color(1, 1, 1, 0)
	var tween = create_tween()
	tween.tween_property(self, "modulate", Color(1, 1, 1, 1), 1.0)


func _on_Cancel_pressed():
	get_tree().quit()


func _on_text_changed(_new_text):
	initalize.disabled = (address.text.length() == 0) || (port.text.length() == 0) || (database.text.length() == 0)


func _on_Initialize_pressed():
	emit_signal("initialize", address.text, port.text, database.text)
