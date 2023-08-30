use ipipe::Pipe;
use std::{thread, time};
use std::sync::{Arc, Mutex};
use std::io::{BufRead, BufReader};
use crate::settings::Settings;
use crate::tester::Tester;

// headers
pub mod settings;
pub mod tester;

static PIPE_FILENAME: &'static str = "status_pipe";


fn main()
{
    let settings = Arc::new(Mutex::new(Settings::new(None)));
    let tester = Tester::new(Arc::clone(&settings));
    let pipe_settings = Arc::clone(&settings);

    thread::spawn(move || pipe_handler(pipe_settings));

    loop {
        tester.test();
        println!("wow");
        {
            // Creating its own scope to prevent holding onto settings
            let settings = settings.lock().unwrap();
            thread::sleep(time::Duration::from_millis(settings.check_interval * 1000));
        }
    }
}

fn pipe_handler(settings : Arc<Mutex<Settings>>){

    let pipe = Pipe::with_name(PIPE_FILENAME).expect("Unable to create pipe");
    println!("Name: {}", pipe.path().display());
    for line in BufReader::new(pipe).lines()
    {
        {
            let settings = settings.lock().unwrap();
            println!("{}", serde_json::to_string_pretty(&settings.services).unwrap());
        }
        println!("{}", line.unwrap());
    }
}
