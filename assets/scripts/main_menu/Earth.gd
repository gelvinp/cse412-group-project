extends CSGSphere

export(float, 0.0, 0.5, 0.01) var rotation_speed
var _rotation_speed = 0.0


func _ready():
	var tween = create_tween()
	tween.tween_property(self, "_rotation_speed", rotation_speed, 1.0)


func _process(delta):
	rotate_y(-delta * _rotation_speed)
