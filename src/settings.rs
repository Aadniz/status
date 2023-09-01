use std::fs;


use serde::{Deserialize, Serialize};
use serde_json::ser::State::Empty;
use serde_json::Value;

const DEFAULT_SETTINGS: Settings = {
    Settings {
        check_interval: 600,
        timeout: 12000,
        services: vec![]
    }
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BoolOrFloat {
    Bool(bool),
    Float(f64)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub(crate) name: String,
    pub(crate) success: BoolOrFloat,
    pub(crate) result: Value,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum ResultOutput {
    String(String),
    Bool(bool),
    Int(i32),
    Float(f32),
    Result(TestResult)
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Service {
    pub name: String,
    pub command: String,
    pub successes: f32,
    pub result: Vec<ResultOutput>
}
impl Service {
    pub fn new(value: &Value) -> Self {

        let name = value.get("name").expect("Missing name value in service").as_str().expect("Name is not a valid string");
        let command = value.get("command").expect("Missing command value in service").as_str().expect("Command is not a valid string");

        Service {
            name: String::from(name),
            command: String::from(command),
            successes: 0.00,
            result: Vec::new()
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Settings {
    pub check_interval: u64,
    pub timeout: u64,
    pub services: Vec<Service>
}

impl Settings {
    pub fn new(path: Option<&str>) -> Self {
        let path : &str = path.unwrap_or("settings.json");
        let file = fs::File::open(path).expect("file should open read only");
        let json: Value = serde_json::from_reader(file).expect("file should be proper JSON");

        let check_interval = json.get("check_interval").and_then(|v| v.as_u64()).unwrap_or_else(|| DEFAULT_SETTINGS.check_interval);
        let timeout = json.get("timeout").and_then(|v| v.as_u64()).unwrap_or_else(|| DEFAULT_SETTINGS.timeout);

        let services_try = json.get("services").and_then(|v| v.as_array());
        let services = match services_try {
            None => {DEFAULT_SETTINGS.services}
            Some(arr) => {
                arr.iter().map(|s| Service::new(s)).collect()
            }
        };

        Settings {
            check_interval,
            timeout,
            services,
        }
    }
}