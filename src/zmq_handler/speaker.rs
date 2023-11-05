use crate::zmq_handler::ZmqHandler;


impl ZmqHandler {
    pub fn print(&mut self, content: String) {
        self.router.send_multipart(&[content], 0).expect("Failed to send message");
    }
}
