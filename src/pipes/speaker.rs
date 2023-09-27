use crate::pipes::PipeHandler;
use std::io::Write;


impl PipeHandler {
    pub fn print(&mut self, content: String){
        self.pipe_out.write_all(format!("{}\n",content).as_bytes()).expect("Ouch owie");
    }
}