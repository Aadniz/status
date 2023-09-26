use std::sync::{Arc, Mutex};
use crate::pipes::PipeHandler;
use std::io::{BufRead, BufReader, Read, Write};
use std::ops::Deref;
use serde::__private::from_utf8_lossy;
use serde_json;

impl PipeHandler {
    pub fn listen(&mut self){

        loop {
            let mut buffer = vec![0; 1024];
            while let Ok(bytes_read) = self.pipe_in.read(&mut buffer) {
                // No clue if this is needed
                if bytes_read == 0 {
                    break;
                }

                // Grabbing the input string
                let input = from_utf8_lossy(&buffer[..bytes_read]).trim().to_string();

                // Getting the settings
                let settings = {
                    let locked_settings = self.settings.lock().unwrap();
                    (*locked_settings).clone()
                };


                // Now that all that is done, we can finally look at what the input actually is
                println!("> {}", input);

                if input == "help" {
                    self.help()
                }else{
                    self.print(serde_json::to_string_pretty(&settings.services).expect("Failed to parse as JSON"));
                }
            }
        }
    }
}