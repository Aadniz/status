use std::{thread, time};
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use crate::pipes::PipeHandler;
use crate::settings::{Service, Settings};
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

    let settings = Settings::new(cli.settings);

    let settings_mutex = Arc::new(Mutex::new(settings));
    let mut pipe = PipeHandler::new(Arc::clone(&settings_mutex));

    thread::Builder::new()
        .name("Listener".to_string())
        .spawn(move || pipe.listen())
        .unwrap();


    // Setting up multithreading handles
    let mut handles = vec![];

    for service in settings.services {
        let service_mutex = Arc::new(Mutex::new(service));
        let handle = thread::spawn(move || test_loop(service_mutex));
        handles.push(handle);
    }


    // Joining the handles (starting the multithreading)
    for handle in handles {
        handle.join().unwrap();
    }
}

fn test_loop(service_mutex : Arc<Mutex<Service>>) {
    let mut interval = 600;

    loop {
        // Locking the resource, and updating it
        {
            let mut locked_service = service_mutex.lock().unwrap();
            let (successes, test_result) = Tester::test(locked_service.clone());
            locked_service.successes = successes;
            locked_service.result = test_result;
        }

        thread::sleep(time::Duration::from_secs(interval));
    }
}


