use crate::pipes::PipeHandler;
use std::io::Read;
use serde::__private::from_utf8_lossy;
use serde_json;
use clap::{Args, Parser, Subcommand};
use crate::settings::Service;

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
    /// Shows the result of the service(s) in a JSON format
    Service (ServiceArgs),

    /// List out all available services
    List,

    /// Show settings
    Settings,
}

#[derive(Args, Debug)]
struct ServiceArgs {
    #[arg(default_value = "all")]
    names: Option<Vec<String>>,
}

impl PipeHandler {

    /// Listens to the input pipe and parses the input.
    ///
    /// This function will continuously read from the input pipe and pass the input to the parser.
    /// It runs in an infinite loop.
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

    /// Parses the input and executes the appropriate commands.
    ///
    /// # Arguments
    ///
    /// * `content` - A string that represents the input to be parsed.
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

                // Was able to parse it
                Ok(v) => v,

                // Wants to show help menu, version menu, or just general error
                Err(e) => {
                    self.print(e.to_string());
                    return;
                }
            };

        match opts.command {
            Commands::Service(args) => self.service_handler(args, settings.services),
            Commands::Settings => self.print(format!("{}", settings)),
            Commands::List => self.print(settings.services.iter().map(|s| s.name.clone()).collect::<Vec<_>>().join(", ").to_string())
        }
    }

    /// Handles the "service" command.
    ///
    /// This function will print the details of the specified services in a JSON format.
    /// If no services are specified, it will print the details of all services.
    ///
    /// # Arguments
    ///
    /// * `args` - The arguments of the "service" command.
    /// * `services` - A vector of `Service` instances that represents the available services.
    fn service_handler(&mut self, args: ServiceArgs, services: Vec<Service>) {
        args.names.as_ref().map(|names| {
            if names.len() == 0 || (names.len() == 1 && names[0] == "all") {
                self.print(serde_json::to_string_pretty(&services).unwrap_or("Failed to parse as JSON".to_string()))
            } else {
                let filtered_services: Vec<_> = services.iter().filter(|s| names.contains(&s.name)).collect();
                if filtered_services.is_empty() {
                    self.print("No services found".to_string());
                } else {
                    self.print(serde_json::to_string_pretty(&filtered_services).unwrap_or("Failed to parse as JSON".to_string()))
                }
            }
        });
    }
}