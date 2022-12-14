extends PanelContainer

signal initialize(address, port, database, user, passw)


onready var address := $MarginContainer/VBoxContainer/VBoxContainer/HBoxContainer/Address
onready var port := $MarginContainer/VBoxContainer/VBoxContainer/HBoxContainer2/Port
onready var database := $MarginContainer/VBoxContainer/VBoxContainer/HBoxContainer3/DatabaseName
onready var username := $MarginContainer/VBoxContainer/VBoxContainer/HBoxContainer4/Username
onready var password := $MarginContainer/VBoxContainer/VBoxContainer/HBoxContainer5/Password
onready var initalize := $MarginContainer/VBoxContainer/HBoxContainer2/Initialize


func _ready():
	modulate = Color(1, 1, 1, 0)
	var tween = create_tween()
	tween.tween_property(self, "modulate", Color(1, 1, 1, 1), 1.0)


func _on_text_changed(_new_text):
	initalize.disabled = (address.text.length() == 0) || (port.text.length() == 0) || (database.text.length() == 0) || (username.text.length() == 0)


func _on_Initialize_pressed():
	emit_signal("initialize", address.text, port.text, database.text, username.text, password.text)


func _on_text_entered(new_text):
	if not initalize.disabled and modulate.a != 0:
		initalize.emit_signal("pressed")


func disable():
	initalize.disabled = true
	initalize.text = "Connecting"
	address.editable = false
	port.editable = false
	database.editable = false
	username.editable = false
	password.editable = false


func enable():
	initalize.disabled = false
	initalize.text = "Initialize"
	address.editable = true
	port.editable = true
	database.editable = true
	username.editable = true
	password.editable = true
