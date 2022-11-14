mod database_connection;

use gdnative::prelude::*;
use database_connection::DatabaseConnection;


fn init(handle: InitHandle)
{
	handle.add_class::<DatabaseConnection>();
}


godot_init!(init);