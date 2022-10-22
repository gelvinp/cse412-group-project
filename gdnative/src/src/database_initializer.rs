mod status;


use gdnative::prelude::*;
use std::sync::{Arc, Mutex};
use status::Status;
use gdnative::export::user_data::MutexData;

type DBResult = Result<(), &'static str>;


#[derive(NativeClass)]
#[inherit(Node)]
#[register_with(Self::register_signals)]
#[user_data(MutexData<DatabaseInitializer>)]
pub struct DatabaseInitializer
{
    region_count: u32,
    status: Arc<Mutex<Status>>,
}


impl DatabaseInitializer
{
	fn new(_base: &Node) -> Self
    {
        DatabaseInitializer
        {
            region_count: 0,
            status: Arc::new(Mutex::new(Status::new())),
        }
    }

    fn register_signals(builder: &ClassBuilder<Self>)
    {
        builder
            .signal("error")
            .with_param("message", VariantType::GodotString)
            .done();

        builder
            .signal("completed")
            .done();
    }
}

#[methods]
impl DatabaseInitializer
{
    #[method]
	fn init_db(&mut self, #[base] owner: &Node)
	{
        match self.import_timepoints()
        {
            Ok(()) => {}
            
            Err(err) =>
            {
                owner.emit_signal("error", &[Variant::new(err)]);
                return
            }
        }

        owner.emit_signal("completed", &[]);
	}

    #[method]
    fn get_status(&mut self, #[base] owner: &Node) -> Dictionary<Unique>
    {
        match self.status.lock()
        {
            Ok(res) =>
            {
                res.dictionary()
            }

            Err(e) =>
            {
                owner.emit_signal("error", &[Variant::new(format!("Failed to aquire the status mutex: {}", e))]);
                
                Dictionary::new()
            }
        }
    }

    #[method]
    fn cancel(&mut self, #[base] owner: &Node)
    {
        match self.status.lock()
        {
            Ok(mut res) =>
            {
                res.cancelled = true;
            }

            Err(e) =>
            {
                owner.emit_signal("error", &[Variant::new(format!("Failed to aquire the status mutex: {}", e))]);
            }
        }
    }
}

impl DatabaseInitializer
{
    fn import_timepoints(&mut self) -> DBResult
    {
        godot_print!("Importing Timepoints");

        Ok(())
    }
}