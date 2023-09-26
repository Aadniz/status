use crate::pipes::PipeHandler;
use std::io::Write;

impl PipeHandler {
    pub fn print(&mut self, content: String){
        self.pipe_out.write_all(content.as_bytes()).expect("ohhh");
    }

    pub fn help(&mut self) {
        self.print("I'm sorry wtf".to_string())
    }
}