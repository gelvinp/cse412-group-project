extends Node

var error_scene := preload("res://assets/scenes/main_menu/Error.tscn")
var error: ErrorDisplay

onready var form := $DBInitForm
onready var camera := $ViewportContainer/Viewport/EarthCamera/Camera

onready var countries := $TimepointForm/MarginContainer/HBoxContainer/OptionButton
onready var columns := $TimepointForm/MarginContainer/HBoxContainer/OptionButton2
onready var earth: CSGSphere = $ViewportContainer/Viewport/EarthCamera/Earth

var timepoint_id := 0
var column := "wp_prec"
var country := "USA"


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
	for country in DbConnection.countries.keys():
		countries.add_item(country)
	var usa_index = DbConnection.countries.keys().find("United States of America")
	countries.select(usa_index)
	_on_OptionButton_item_selected(usa_index)
	columns.add_item("Precipitation")
	columns.add_item("Minimum Temp")
	columns.add_item("Maximum Temp")
	$TimepointForm.visible = true


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


func _on_OptionButton_item_selected(index):
	var coords = DbConnection.countries.values()[index]
	print("Selected ", DbConnection.countries.keys()[index])
	print("Coords are ", coords[0], ", ", coords[1])
	
	country = coords[2]
	regen_texture()

	var coord0 = (((coords[0] as float) / 8640.0) * 2 * PI + ((3.0/2.0) * PI)) * -1
	var coord1 =  (((coords[1] as float) / 4320.0) * PI - PI/2) * -1
	camera.transform.origin.x = 3 * cos(coord1) * cos(coord0)
	camera.transform.origin.y = 3 * sin(coord1)
	camera.transform.origin.z = 3 * cos(coord1) * sin(coord0) 
	print(coord0, ", ", coord1)
	camera.look_at(Vector3(0, 0, 0), Vector3.UP)


const months := ["Jan", "Apr", "Jul", "Oct"]

func _on_HSlider_value_changed(value):
	var month = months[value as int % 4]
	var year = 1960 + ((value as int / 4) * 10)
	
	$TimepointForm/MarginContainer/HBoxContainer/Label.text = "%s %s's" % [month, year]


func _on_HSlider_drag_ended(value_changed):
	if value_changed:
		timepoint_id = $TimepointForm/MarginContainer/HBoxContainer/HSlider.value
		regen_texture()


func regen_texture():
	var texture
	
	if country.empty():
		texture = DbConnection.connection.get_texture_for_timepoint(timepoint_id, column)
	else:
		texture = DbConnection.connection.get_texture_for_timepoint_country(timepoint_id, column, country)
	
	(earth.material as SpatialMaterial).albedo_texture = texture
	(earth.material.next_pass as ShaderMaterial).set_shader_param("color_strength", 1.0)
	(earth.material as SpatialMaterial).flags_unshaded = true


const column_options := ["wp_prec", "wp_tmin", "wp_tmax"]

func _on_OptionButton2_item_selected(index):
	column = column_options[index]
	regen_texture()
