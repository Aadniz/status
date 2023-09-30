use std::{thread, time};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use crate::pipes::PipeHandler;
use crate::settings::Settings;
use crate::tester::Tester;
use clap::{Parser};


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

    // Starting listening thread
    thread::Builder::new()
        .name("Listener".to_string())
        .spawn(move || pipe.listen())
        .unwrap();


    // Setting up multithreading handles
    let mut handles : Vec<JoinHandle<()>> = vec![];

    // Looks a bit cryptic, this was needed to allow shared memory
    let services_mutex = Arc::clone(&settings_mutex);
    for i in 0..services_mutex.lock().unwrap().services.len() {
        let services_mutex = Arc::clone(&services_mutex);
        let handle = thread::spawn(move || test_loop(services_mutex, i));
        handles.push(handle);
    }


    // Joining the handles (starting the multithreading)
    for handle in handles {
        handle.join().unwrap();
    }
}

/// Running a single command/test in its independent loop
///
/// # Arguments
///
/// * `services_mutex` - An Arc Mutex that contains the settings.
/// * `index` - The index of the service to be tested.
fn test_loop(services_mutex : Arc<Mutex<Settings>>, index: usize) {
    loop {
        let service = { services_mutex.lock().unwrap().services[index].clone() };
        let interval = service.interval;
        let (successes, test_result) = Tester::test(service);

        // Locking the resource, and updating it
        {
            let mut locked_settings = services_mutex.lock().unwrap();
            locked_settings.services[index].successes = successes;
            locked_settings.services[index].result = test_result;
        }
        thread::sleep(time::Duration::from_secs(interval));
    }
}