use crate::zmq_handler::ZmqHandler;
use serde_json;
use clap::{Args, Parser, Subcommand};
use crate::settings::{ResultOutput, Service};

/// Status daemon written in rust.
/// Check services output
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

    /// Only show errors
    #[arg(long = "errors")]
    errors: bool,
}

impl ZmqHandler {

    /// Continuously read from the ZMQ and return the appropriate string information.
    /// It runs in an infinite loop.
    pub fn listen(&mut self) {
        loop {

            // Receive a multipart message from the subscriber
            let msg_result = self.router.recv_multipart(0);

            match msg_result {
                Ok(msg) => {
                    let (id, input) = (msg[0].clone(), String::from_utf8(msg[1].clone()));
                    match input {
                        Ok(input_str) => {
                            // Now that all that is done, we can finally look at what the input actually is
                            println!("> {}", input_str);

                            // Pass the input to the parser and get the reply
                            let reply = self.parser(input_str);

                            // Send reply back to client
                            self.router.send_multipart(&[id, reply.into_bytes()], 0).unwrap();
                        },
                        Err(_) => eprintln!("Received invalid UTF-8 data"),
                    }
                },
                Err(e) => eprintln!("Failed to receive message: {}", e),
            }
        }
    }

    /// Parses the input and executes the appropriate commands.
    ///
    /// # Arguments
    ///
    /// * `content` - A string that represents the input to be parsed.
    fn parser(&mut self, content: String) -> String {

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
                return e.to_string();
            }
        };

        match opts.command {
            Commands::Service(args) => self.service_handler(args, settings.services),
            Commands::Settings => format!("{}", settings),
            Commands::List => settings.services.iter().map(|s| s.name.clone()).collect::<Vec<_>>().join(", ").to_string()
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
    fn service_handler(&mut self, args: ServiceArgs, services: Vec<Service>) -> String {
        let short = args.short;
        let errors = args.errors;
        let names = args.names.as_ref();

        // Check if names are specified
        let mut services_to_print = match names {
            Some(names) if !names.is_empty() && names[0] != "all" => {
                services.into_iter().filter(|service| names.contains(&service.name)).collect()
            },
            _ => services,
        };

        // If no matching services are found, print a message
        if services_to_print.is_empty() {
            return "No services found".to_string();
        }

        if errors {
            services_to_print.retain(|service| service.successes != 1.0);
            for s in &mut services_to_print {
                match &mut s.result {
                    ResultOutput::Result(r) => {
                        r.retain(|test_result| test_result.success != 1.0);
                    }
                    _ => {}
                }
            }
        }

        // Prepare the output
        let output= if short {
            // If `short` option is specified, manually construct JSON excluding the `result` field
            let short_services: Vec<_> = services_to_print.iter().map(|service| {
                serde_json::json!({
                        "name": service.name,
                        "command": service.command,
                        "args": service.args,
                        "interval": service.interval,
                        "timeout": service.timeout,
                        "successes": service.successes,
                        "pause_on_no_internet": service.pause_on_no_internet,
                    })
            }).collect();
            serde_json::to_string_pretty(&short_services)
        } else {
            serde_json::to_string_pretty(&services_to_print)
        };

        // Print the output
        return output.unwrap_or("Failed to parse as JSON".to_string());
    }
}