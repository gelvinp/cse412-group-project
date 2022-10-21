extends Node


func _on_DBInitForm_initialize(address, port, database):
	var db_init = DatabaseInitializer.new()
	db_init.init_db()
