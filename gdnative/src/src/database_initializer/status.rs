use gdnative::prelude::*;

static STAGES: [&str; 5] = ["Starting up", "Importing time points", "Importing regions", "Importing countries", "Importing weather points"];


pub struct Status
{
    pub current_stage: &'static str,
    pub current_progress: u32,
    pub total_work: u32,
    pub discrete_progress: bool,
    pub cancelled: bool,
}

impl Status
{
    pub fn new() -> Self
    {
        Status
        {
            current_stage: STAGES[0],
            current_progress: 0,
            total_work: 0,
            discrete_progress: false,
            cancelled: false,
        }
    }

    pub fn dictionary(&self) -> Dictionary<Unique>
    {
        let dict = Dictionary::new();

        dict.insert("stage", self.current_stage);
        dict.insert("discrete", self.discrete_progress);

        if self.discrete_progress
        {
            dict.insert("progress", self.current_progress);
            dict.insert("total", self.total_work);
        }

        dict
    }
}