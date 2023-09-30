use crate::pipes::PipeHandler;
use std::io::Write;


impl PipeHandler {

    /// Writes a string to the output pipe.
    ///
    /// This function will append a newline character to the string and write it to the output pipe.
    ///
    /// # Arguments
    ///
    /// * `content` - The string to be written to the output pipe.
    ///
    /// # Panics
    ///
    /// This function will panic if it fails to write to the output pipe.
    pub fn print(&mut self, content: String){
        self.pipe_out.write_all(format!("{}\n",content).as_bytes()).expect("Ouch owie");
    }
}