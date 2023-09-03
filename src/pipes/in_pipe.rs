use std::io::{BufRead, BufReader};
use std::sync::{Arc, Mutex};
use ipipe::Pipe;
use crate::pipes::out_pipe::OutPipe;
use crate::pipes::{PIPE_IN_FILENAME};
use crate::settings::Settings;

pub struct InPipe {
    pipe_in: Pipe,
    settings: Arc<Mutex<Settings>>
}
impl InPipe {
    pub fn new(settings : Arc<Mutex<Settings>>) -> Self {
        let pipe_in = Pipe::with_name(PIPE_IN_FILENAME).expect(format!("Unable to create pipe: {}", PIPE_IN_FILENAME).as_str());

        println!("In Pipe:  {}", pipe_in.path().display());

        InPipe {
            pipe_in,
            settings
        }
    }

    pub fn listen(mut self){
        for line in BufReader::new(self.pipe_in).lines()
        {
            println!("> {}", line.unwrap());
            let settings = { self.settings.lock().unwrap() };
            OutPipe::print(serde_json::to_string_pretty(&settings.services).expect("Failed to parse as JSON"));
        }
    }
}