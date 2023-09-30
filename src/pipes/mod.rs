mod speaker;
mod listen;

use std::sync::{Arc, Mutex};
use crate::settings::Settings;
use ipipe::Pipe;

const PIPE_IN_FILENAME : &str = "status_in_pipe";
const PIPE_OUT_FILENAME : &str = "status_out_pipe";

/// The `PipeHandler` struct represents a handler for two named pipes: an input pipe and an output pipe.
///
/// # Fields
///
/// * `pipe_in` - The input pipe.
/// * `pipe_out` - The output pipe.
/// * `settings` - A shared reference to the settings.
pub struct PipeHandler {
    pipe_in: Pipe,
    pipe_out: Pipe,
    settings: Arc<Mutex<Settings>>
}

impl PipeHandler {
    /// Creates a new `PipeHandler` instance.
    ///
    /// This function will create two pipes with predefined names and print their paths to the console.
    ///
    /// # Arguments
    ///
    /// * `settings` - A shared reference to the settings.
    pub fn new(settings : Arc<Mutex<Settings>>) -> Self {

        let pipe_in = Pipe::with_name(PIPE_IN_FILENAME).expect(format!("Unable to create pipe: {}", PIPE_IN_FILENAME).as_str());
        let pipe_out = Pipe::with_name(PIPE_OUT_FILENAME).expect(format!("Unable to create pipe: {}", PIPE_OUT_FILENAME).as_str());

        println!("In Pipe:\t{}", pipe_in.path().display());
        println!("Out Pipe:\t{}", pipe_out.path().display());

        PipeHandler {
            pipe_in,
            pipe_out,
            settings
        }
    }
}