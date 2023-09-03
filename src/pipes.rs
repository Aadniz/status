use std::sync::{Arc, Mutex};
use crate::settings::Settings;
use std::io::{BufRead, BufReader, Write};
use ipipe::Pipe;

const PIPE_IN_FILENAME : &str = "status_in_pipe";
const PIPE_OUT_FILENAME : &str = "status_out_pipe";

pub struct PipeHandler {
    pipe_in: Pipe,
    pipe_out: Pipe,
    settings: Arc<Mutex<Settings>>
}


impl PipeHandler {
    pub fn new(settings : Arc<Mutex<Settings>>) -> Self {
        let pipe_in = Pipe::with_name(PIPE_IN_FILENAME).expect(format!("Unable to create pipe: {}", PIPE_IN_FILENAME).as_str());
        let pipe_out = Pipe::with_name(PIPE_OUT_FILENAME).expect(format!("Unable to create pipe: {}", PIPE_OUT_FILENAME).as_str());

        println!("In Pipe:  {}", pipe_in.path().display());
        println!("Out Pipe: {}", pipe_out.path().display());

        PipeHandler {
            pipe_in,
            pipe_out,
            settings
        }
    }

    pub fn listen(mut self){
        for line in BufReader::new(self.pipe_in).lines()
        {
            println!("> {}", line.unwrap());
            let settings = { self.settings.lock().unwrap() };
            //self.print(serde_json::to_string_pretty(&settings.services).expect("Failed to parse as JSON"));
            self.pipe_out.write_all(serde_json::to_string_pretty(&settings.services).expect("Failed to parse as JSON").as_bytes()).expect("Failed to write to pipe");
        }
    }


    pub fn print(&mut self, content: String){
        self.pipe_out.write_all(content.as_bytes()).expect("ohhh");
    }
}