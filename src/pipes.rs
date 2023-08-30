use std::fmt;
use std::fmt::{Display, format};
use ipipe::Pipe;
use std::sync::{Arc, Mutex};
use crate::settings::Settings;
use std::io::{BufRead, BufReader, Write};


static PIPE_IN_FILENAME: &'static str = "status_in_pipe";
static PIPE_OUT_FILENAME: &'static str = "status_out_pipe";


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

    pub fn listen(self){
        for line in BufReader::new(self.pipe_in).lines()
        {
            println!("> {}", line.unwrap());
            let settings = { self.settings.lock().unwrap() };
            //self.print(serde_json::to_string_pretty(&settings.services).unwrap());
        }
    }


    pub fn print(mut self, content: String){
        self.pipe_out.write_all(content.as_bytes());
    }
}