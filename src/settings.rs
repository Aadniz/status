use std::{fmt, fs};
use serde::{Deserialize, Serialize};
use serde_json::Value;

const DEFAULT_SETTINGS: Settings = {
    Settings {
        check_interval: 600,
        timeout: 60.0,
        services: vec![]
    }
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub(crate) name: String,
    pub(crate) success: f64,
    pub(crate) result: Value,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum ResultOutput {
    String(String),
    Bool(bool),
    Int(i32),
    Float(f32),
    Result(Vec<TestResult>)
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Service {
    pub name: String,
    pub command: String,
    pub successes: f64,
    pub result: ResultOutput
}
impl Service {
    pub fn new(value: &Value) -> Self {

        let name = value.get("name").expect("Missing name value in service").as_str().expect("Name is not a valid string");
        let command = value.get("command").expect("Missing command value in service").as_str().expect("Command is not a valid string");

        Service {
            name: String::from(name),
            command: String::from(command),
            successes: 0.00,
            result: ResultOutput::Bool(false)
        }
    }
}
impl fmt::Display for Service {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Name: {}, Command: {}", self.name, self.command)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Settings {
    pub check_interval: u64,
    pub timeout: f64,
    pub services: Vec<Service>
}

impl Settings {
    pub fn new(path: Option<&str>) -> Self {
        let path : &str = path.unwrap_or("settings.json");
        let file = fs::File::open(path).expect("file should open read only");
        let json: Value = serde_json::from_reader(file).expect("file should be proper JSON");

        let check_interval = json.get("check_interval").and_then(|v| v.as_u64()).unwrap_or_else(|| DEFAULT_SETTINGS.check_interval);
        let timeout = json.get("timeout").and_then(|v| v.as_f64()).unwrap_or_else(|| DEFAULT_SETTINGS.timeout);

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

impl fmt::Display for Settings {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "Check Interval: {}\n\
               Timeout: {}\n\
               Services:\n{}\n",
               self.check_interval,
               self.timeout,
               self.services.iter().map(|s| s.to_string()).collect::<Vec<_>>().join("\n"))
    }
}