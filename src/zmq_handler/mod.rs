use std::sync::{Arc, Mutex};
use zmq::{Context, Socket, ROUTER};

use crate::settings::Settings;

mod listen;
mod speaker;

/// The `ZmqHandler` struct, which holds a ZeroMQ ROUTER socket and the application settings.
pub struct ZmqHandler {
    /// The ZeroMQ ROUTER socket for sending and receiving messages.
    router: Socket,
    /// The application settings, wrapped in an Arc and Mutex for thread safety.
    settings: Arc<Mutex<Settings>>,
}

impl ZmqHandler {
    /// Constructs a new `ZmqHandler`.
    ///
    /// This function takes the application settings as a parameter, creates a new ZeroMQ context and a ROUTER socket,
    /// and binds the socket to the specified protocol and port from the settings.
    ///
    /// # Arguments
    ///
    /// * `settings` - An Arc<Mutex<Settings>> that contains the application settings.
    ///
    /// # Returns
    ///
    /// * A new `ZmqHandler` with the created socket and the provided settings.
    pub fn new(settings: Arc<Mutex<Settings>>) -> Self {
        // Extract the protocol and port from the settings
        let (protocol, port) = {
            let settings = settings.lock().unwrap().clone();
            (settings.protocol, settings.port)
        };

        // Create a new ZeroMQ context
        let context = Context::new();

        // Create a new ZeroMQ ROUTER socket
        let socket = context.socket(ROUTER).unwrap();

        // Bind the socket to the specified protocol and port
        socket
            .bind(&*format!("{}://*:{}", protocol, port))
            .expect("Unable to bind socket");

        // Return a new ZmqHandler with the created socket and the provided settings
        ZmqHandler {
            router: socket,
            settings,
        }
    }
}
