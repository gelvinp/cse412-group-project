use gdnative::prelude::*;


#[derive(NativeClass)]
#[inherit(Node)]
pub struct DatabaseInitializer;

fn init(handle: InitHandle)
{
	handle.add_class::<DatabaseInitializer>();
}

godot_init!(init);

impl DatabaseInitializer
{
	fn new(_base: &Node) -> Self { DatabaseInitializer }
}

#[methods]
impl DatabaseInitializer
{
	#[method]
	fn init_db(&self)
	{
		godot_print!("Hello, Godot!");
	}
}