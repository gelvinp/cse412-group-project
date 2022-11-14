extends CSGSphere

export(float, 0.0, 0.5, 0.01) var rotation_speed
var _rotation_speed = 0.0

var data_texture: ImageTexture


func _ready():
	#data_texture = ImageTexture.create(2700, 1350)
	var tween = create_tween()
	tween.tween_property(self, "_rotation_speed", rotation_speed, 1.0)


func _process(delta):
	if DbConnection.countries.size() == 0:
		rotate_y(-delta * _rotation_speed)
	else:
		var tween = create_tween()
		tween.set_ease(Tween.EASE_IN_OUT)
		tween.set_trans(Tween.TRANS_SINE)
		tween.tween_property(self, "rotation:y", 0.0, 0.6)
		set_process(false)
