use std::io::Write;
use ipipe::Pipe;
use crate::pipes::PIPE_OUT_FILENAME;

const PIPE_OUT: Pipe = Pipe::with_name(PIPE_OUT_FILENAME).expect(format!("Unable to create pipe: {}", PIPE_OUT_FILENAME).as_str());

pub struct OutPipe {

}

impl OutPipe {
    pub fn print(content: String){
        PIPE_OUT.write_all(content.as_bytes()).expect("TODO: panic message");
    }
}