use crate::pipes::PipeHandler;
use std::io::Read;
use serde::__private::from_utf8_lossy;
use serde_json;
use std::{thread, time};
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

#[derive(Args)]
struct ServiceArgs {
    /// The name of the service to show information from
    #[arg(default_value = "all")]
    names: Option<Vec<String>>,

    /// Shorten the result
    #[arg(long = "short")]
    short: bool,
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

                // Should help prevent race condition lol
                thread::sleep(time::Duration::from_millis(200));
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
        let short = args.short;
        let names = args.names.as_ref();

        // Check if names are specified
        if let Some(names) = names {
            let mut services_to_print = Vec::new();

            // If "all" is specified or no names are specified, add all services
            if names.len() == 0 || (names.len() == 1 && names[0] == "all") {
                services_to_print = services.iter().collect();
            } else {
                // Otherwise, add only the services with the specified names
                for service in &services {
                    if names.contains(&service.name) {
                        services_to_print.push(service);
                    }
                }
            }

            // If no matching services are found, print a message
            if services_to_print.is_empty() {
                self.print("No services found".to_string());
                return;
            }

            // Prepare the output
            let output;
            if short {
                // If `short` option is specified, manually construct JSON excluding the `result` field
                let mut short_services = Vec::new();
                for service in &services_to_print {
                    let short_service = serde_json::json!({
                    "name": service.name,
                    "command": service.command,
                    "args": service.args,
                    "interval": service.interval,
                    "timeout": service.timeout,
                    "successes": service.successes,
                    "pause_on_no_internet": service.pause_on_no_internet,
                });
                    short_services.push(short_service);
                }
                output = serde_json::to_string_pretty(&short_services);
            } else {
                output = serde_json::to_string_pretty(&services_to_print);
            }

            // Print the output
            self.print(output.unwrap_or("Failed to parse as JSON".to_string()));
        }
    }
}