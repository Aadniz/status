use std::sync::{Arc, Mutex};
use crate::pipes::PipeHandler;
use std::io::{BufRead, BufReader, Read, Write};
use std::ops::Deref;
use serde::__private::from_utf8_lossy;
use serde_json;
use clap::{Args, Parser, Subcommand};
use crate::settings::{Service, Settings};

/// Status daemon written in rust.
/// Check services output and communicate via named pipe
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Specify service(s) to do action to
    Service (ServiceArgs),

    /// Show settings
    Settings,
}

#[derive(Args, Debug)]
struct ServiceArgs {
    #[arg(default_value = "all")]
    names: Option<Vec<String>>,
}

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

                // Now that all that is done, we can finally look at what the input actually is
                println!("> {}", input);

                // Pass the input to the parser
                self.parser(input);
            }
        }
    }


    fn parser(&mut self, content: String) {

        // Getting the settings
        let settings = {
            let locked_settings = self.settings.lock().unwrap();
            (*locked_settings).clone()
        };

        let opts = match Cli::try_parse_from(format!("{} {}", std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("status")).display().to_string(), content)
            .to_string()
            .split_whitespace()
            .map(String::from)
            .map(|s| s.trim().to_string())
            .filter(|s|!s.is_empty())
            .collect::<Vec<String>>()) {
            Ok(v) => v,
            Err(e) => {
                self.print(e.to_string());
                return;
            }
        };

        match opts.command {
            Commands::Service(args) => self.service_handler(args, settings.services),
            Commands::Settings => self.print(format!("{}", settings)),
        }
    }

    fn service_handler(&mut self, args: ServiceArgs, services: Vec<Service>) {
        args.names.as_ref().map(|names| {
            if names.len() == 0 || (names.len() == 1 && names[0] == "all") {
                Some(self.print(serde_json::to_string_pretty(&services).expect("Failed to parse as JSON")))
            } else {
                None
            }
        });
    }
}