use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub enum Protocol {
    Tcp,
    Ipc,
    Pgm,
    Epgm,
}

impl Protocol {
    pub fn as_str(&self) -> &'static str {
        match *self {
            Protocol::Tcp => "tcp",
            Protocol::Ipc => "ipc",
            Protocol::Pgm => "pgm",
            Protocol::Epgm => "epgm",
        }
    }

    pub fn from_str(protocol: &str) -> Option<Protocol> {
        match protocol {
            "tcp" => Some(Protocol::Tcp),
            "ipc" => Some(Protocol::Ipc),
            "pgm" => Some(Protocol::Pgm),
            "epgm" => Some(Protocol::Epgm),
            _ => None,
        }
    }
}

impl Display for Protocol {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}