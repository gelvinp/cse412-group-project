use gdnative::prelude::*;

static STAGES: [&str; 9] = ["Starting up", "Importing regions", "Processing regions", "Importing time points", "Processing time points", "Importing weather points", "Processing weather points", "Importing countries", "Processing countries"];


pub struct Status
{
    pub current_stage: &'static str,
    pub current_progress: u32,
    pub total_work: u32,
    pub discrete_progress: bool,
    pub cancelled: bool,
    pub completed: bool,
    pub error: Option<String>,
    pub working_space: u32,
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
            completed: false,
            error: None,
            working_space: 0,
        }
    }

    pub fn dictionary(&self) -> Dictionary<Unique>
    {
        let dict = Dictionary::new();

        dict.insert("completed", self.completed);
        
        if self.completed
        {
            return dict;
        }

        match &self.error
        {
            Some(error) =>
            {
                dict.insert("error", error);
                return dict;
            }

            None => {}
        }

        dict.insert("stage", self.current_stage);
        dict.insert("discrete", self.discrete_progress);

        if self.discrete_progress
        {
            dict.insert("progress", self.current_progress);
            dict.insert("total", self.total_work);
        }

        dict
    }

    pub fn set_stage(&mut self, stage: usize)
    {
        self.current_stage = STAGES[stage];
    }
}