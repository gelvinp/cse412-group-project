extends CSGSphere

export(float, 0.0, 0.5, 0.01) var rotation_speed
var _rotation_speed = 0.0

var data_texture = ImageTexture.new()
var _spinning = true


func _ready():
	var tween = create_tween()
	tween.tween_property(self, "_rotation_speed", rotation_speed, 1.0)
#	var image = Image.new()
#	image.create(8640, 4320, false, Image.FORMAT_L8)
#	image.fill(0)
	
#	image.lock()
#	for i in 8640:
#		var value = float(i % 864) / 863
#
#		for j in 4320:
#			image.set_pixel(i, j, Color(value, value, value))
#	image.unlock()
##
#	data_texture.create_from_image(image, ImageTexture.FLAGS_DEFAULT)
#
#	(material as SpatialMaterial).albedo_texture = data_texture
#	(material.next_pass as ShaderMaterial).set_shader_param("passthrough", false)
#	(material.next_pass as ShaderMaterial).set_shader_param("high", Color(0.0, 1.0, 0.0))
	


func _process(delta):
	if DbConnection.countries.size() == 0:
		rotate_y(-delta * _rotation_speed)
	elif _spinning:
		var tween = create_tween()
		tween.set_ease(Tween.EASE_IN_OUT)
		tween.set_trans(Tween.TRANS_SINE)
		tween.tween_property(self, "rotation:y", 0.0, 0.6)
		_spinning = false
