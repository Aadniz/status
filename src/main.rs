use std::{thread, time};
use std::sync::{Arc, Mutex};
use crate::pipes::PipeHandler;
use crate::settings::Settings;
use crate::tester::Tester;
use clap::{Args, Parser, Subcommand};


// headers
pub mod settings;
pub mod tester;
pub mod pipes;

/// Status daemon written in rust.
/// Check services output and communicate via named pipe
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to json settings file
    settings: Option<String>,
}

fn main()
{
    let cli = Cli::parse();

    let settings = Arc::new(Mutex::new(Settings::new(cli.settings)));
    let tester = Tester::new(Arc::clone(&settings));
    let mut pipe = PipeHandler::new(Arc::clone(&settings));

    thread::Builder::new()
        .name("Listener".to_string())
        .spawn(move || pipe.listen())
        .unwrap();


    // Setting up multithreading handles
    let mut handles = vec![];

    {
        let settings = settings.lock().unwrap().clone();
        for service in settings.services {
            let tester_clone = tester.clone();
            let handle = thread::spawn(move || {
                loop {
                    println!("We are here now");
                    tester_clone.test(&service);
                    thread::sleep(time::Duration::from_secs(service.interval));
                }
            });
            handles.push(handle);
        }
    }

    // Joining the handles (starting the multithreading)
    for handle in handles {
        handle.join().unwrap();
    }
}


