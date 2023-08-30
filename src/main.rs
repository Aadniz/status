use ipipe::Pipe;
use std::{thread, time};
use std::io::{BufRead, BufReader};
use crate::settings::Settings;
use crate::tester::Tester;

// headers
pub mod settings;
pub mod tester;

const PIPE_FILENAME: &str = "status_pipe";


fn main()
{
    let settings = Settings::new(None);
    let tester = Tester::new();

    thread::spawn(move || pipe_handler());
    loop {
        println!("{:?}", settings);
        thread::sleep(time::Duration::from_millis(settings.check_interval * 1000));
    }
}

fn pipe_handler(){

    let pipe = Pipe::with_name(PIPE_FILENAME).expect("Unable to create pipe");
    println!("Name: {}", pipe.path().display());
    for line in BufReader::new(pipe).lines()
    {
        println!("{}", line.unwrap());
    }
}