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

    loop {
        tester.test();
        // TODO: single-threaded service checker here

        let interval : u64 = {
            // Creating its own scope to prevent holding onto settings
            let settings = settings.lock().unwrap();
            settings.interval
        };
        thread::sleep(time::Duration::from_millis(interval * 1000));
    }
}


