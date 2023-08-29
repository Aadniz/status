use std::fs;
use ipipe::Error;


use serde::{Deserialize, Serialize};
use serde_json::Value;

const DEFAULT_SETTINGS: Settings = {
    Settings {
        check_interval: 600,
        max_ping: 7000,
        command_timeout: 12000,
        services: vec![]
    }
};

#[derive(Deserialize, Serialize, Debug)]
struct Service {
    name: String,
    command: String
}
impl Service {
    pub fn new(value: &Value) -> Self {

        let name = value.get("name").expect("Missing name value in service").as_str().expect("Name is not a valid string");
        let command = value.get("command").expect("Missing command value in service").as_str().expect("Command is not a valid string");

        Service {
            name: String::from(name),
            command: String::from(command)
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Settings {
    check_interval: u64,
    max_ping: u64,
    command_timeout: u64,
    services: Vec<Service>
}

impl Settings {
    pub fn new(path: Option<&str>) -> Self {
        let path : &str = path.unwrap_or("settings.json");
        let file = fs::File::open(path).expect("file should open read only");
        let json: Value = serde_json::from_reader(file).expect("file should be proper JSON");

        let check_interval = json.get("check_interval").and_then(|v| v.as_u64()).unwrap_or_else(|| DEFAULT_SETTINGS.check_interval);
        let max_ping = json.get("max_ping").and_then(|v| v.as_u64()).unwrap_or_else(|| DEFAULT_SETTINGS.max_ping);
        let command_timeout = json.get("command_timeout").and_then(|v| v.as_u64()).unwrap_or_else(|| DEFAULT_SETTINGS.command_timeout);

        let services_try = json.get("services").and_then(|v| v.as_array());
        let services = match services_try {
            None => {DEFAULT_SETTINGS.services}
            Some(arr) => {
                arr.iter().map(|s| Service::new(s)).collect()
            }
        };

        Settings {
            check_interval,
            max_ping,
            command_timeout,
            services,
        }
    }
}