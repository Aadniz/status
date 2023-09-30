use std::{fmt, fs};
use serde::{Deserialize, Serialize};
use serde_json::Value;

const DEFAULT_SETTINGS: Settings = {
    Settings {
        interval: 600,
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
    pub args: Option<Vec<String>>,
    pub interval: u64,
    pub timeout: f64,
    pub successes: f64,
    pub result: ResultOutput
}
impl Service {
    pub fn new(value: &Value, settings: Settings) -> Self {

        let name = value.get("name").expect("Missing name value in service").as_str().expect("Name is not a valid string");
        let command = value.get("command").expect("Missing command value in service").as_str().expect("Command is not a valid string");
        let args : Option<Vec<String>> = value.get("args").map(|v| {
            v.as_array()
                .expect("args is not a valid array")
                .iter()
                .map(|s| s.as_str().expect("arg is not a valid string").to_string())
                .collect()
        });
        let interval = value.get("interval").and_then(|v| v.as_u64()).unwrap_or(settings.interval);
        let timeout = value.get("timeout").and_then(|v| v.as_f64()).unwrap_or(settings.timeout);

        Service {
            name: String::from(name),
            command: String::from(command),
            args,
            interval,
            timeout,
            successes: 0.00,
            result: ResultOutput::Bool(false)
        }
    }
}

impl fmt::Display for Service {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "name: {}, command: {}, interval: {}, timeout: {}", self.name, self.command, self.interval, self.timeout)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Settings {
    pub interval: u64,
    pub timeout: f64,
    pub services: Vec<Service>
}

impl Settings {

    fn bare(json: Value) -> Self {

        let interval = json.get("interval").and_then(|v| v.as_u64()).unwrap_or_else(|| DEFAULT_SETTINGS.interval);
        let timeout = json.get("timeout").and_then(|v| v.as_f64()).unwrap_or_else(|| DEFAULT_SETTINGS.timeout);
        let services : Vec<Service> = vec![];

        // Do NOT create the service here!
        assert!(services.is_empty());


        Settings {
            interval,
            timeout,
            services
        }
    }

    pub fn new(path: Option<String>) -> Self {

        let path : String = path.unwrap_or("settings.json".to_string());

        println!("Settings path:\t{}", path);

        let file = match fs::File::open(path) {
            Ok(file) => file,
            Err(error) => panic!("Unable to open the file: {}", error),
        };
        let json: Value = serde_json::from_reader(file).expect("file should be proper JSON");

        // Here we create the bare-bone settings. Needed in order to reference parent JSON in services
        let settings = Settings::bare(json.clone());

        let services_try = json.get("services").and_then(|v| v.as_array());
        let services = match services_try {
            None => {DEFAULT_SETTINGS.services}
            Some(arr) => {
                arr.iter().map(|s| Service::new(s, settings.clone())).collect()
            }
        };

        Settings {
            interval: settings.interval,
            timeout: settings.timeout,
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
               self.interval,
               self.timeout,
               self.services.iter().map(|s| s.to_string()).collect::<Vec<_>>().join("\n"))
    }
}