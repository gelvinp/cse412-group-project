mod database_initializer;

use gdnative::prelude::*;
use database_initializer::DatabaseInitializer;


fn init(handle: InitHandle)
{
	handle.add_class::<DatabaseInitializer>();
}


godot_init!(init);