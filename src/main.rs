use ipipe::Pipe;
use std::{thread, time};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::process::exit;
use crate::settings::Settings;

// headers
pub mod settings;

const PIPE_FILENAME: &str = "status_pipe";


fn main()
{
    let settings = Settings::new(None);

    thread::spawn(move || pipe_handler());
    loop {
        println!("{:?}", settings);
        thread::sleep(time::Duration::from_millis(600 * 1000));
    }
}

fn pipe_handler(){

    let mut pipe = Pipe::with_name(PIPE_FILENAME).expect("Unable to create pipe");
    println!("Name: {}", pipe.path().display());
    for line in BufReader::new(pipe).lines()
    {
        println!("{}", line.unwrap());
    }
}