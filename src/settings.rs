use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fmt, fs};

use crate::service::Service;
use crate::utils::Protocol;

fn default_settings() -> Settings {
    Settings {
        protocol: Protocol::Tcp,
        port: 5747,
        interval: 600,
        timeout: 60.0,
        pause_on_no_internet: false,
        services: vec![],
    }
}

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
    Null,
    Result(Vec<TestResult>),
}

/// Global settings
#[derive(Deserialize, Serialize, Clone)]
pub struct Settings {
    pub protocol: Protocol,
    pub port: u16,
    pub interval: u64,
    pub timeout: f64,
    pub pause_on_no_internet: bool,
    pub services: Vec<Service>,
}

impl Settings {
    /// Creates a new `Settings` instance without creation of services.
    ///
    /// # Arguments
    ///
    /// * `json` - A `Value` that contains the settings.
    fn bare(json: Value) -> Self {
        let default_settings = default_settings();

        let protocol = json
            .get("protocol")
            .and_then(|v| v.as_str())
            .and_then(|s| Protocol::from_str(s))
            .unwrap_or(default_settings.protocol);
        let port = json
            .get("port")
            .and_then(|v| v.as_u64())
            .unwrap_or_else(|| default_settings.port as u64) as u16;
        let interval = json
            .get("interval")
            .and_then(|v| v.as_u64())
            .unwrap_or_else(|| default_settings.interval);
        let timeout = json
            .get("timeout")
            .and_then(|v| v.as_f64())
            .unwrap_or_else(|| default_settings.timeout);
        let pause_on_no_internet = json
            .get("pause_on_no_internet")
            .and_then(|v| v.as_bool())
            .unwrap_or_else(|| default_settings.pause_on_no_internet);
        let services: Vec<Service> = vec![];

        // Do NOT create the service here!
        assert!(services.is_empty());

        Settings {
            protocol,
            port,
            interval,
            timeout,
            pause_on_no_internet,
            services,
        }
    }

    /// Creates a new `Settings` instance given an optional settings path.
    ///
    /// # Arguments
    ///
    /// * `path` - An optional string that represents the path to the JSON file. If no path is provided, "settings.json" is used by default.
    pub fn new(path: Option<String>) -> Self {
        let path: String = path.unwrap_or("settings.json".to_string());

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
            None => default_settings().services,
            Some(arr) => arr
                .iter()
                .map(|s| Service::new(s, settings.clone()))
                .collect(),
        };

        Settings {
            protocol: settings.protocol,
            port: settings.port,
            interval: settings.interval,
            timeout: settings.timeout,
            pause_on_no_internet: settings.pause_on_no_internet,
            services,
        }
    }
}

impl fmt::Display for Settings {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Check Interval: {}\n\
               Timeout: {}\n\
               Skip with no internet: {}\n\
               Services:\n{}\n",
            self.interval,
            self.timeout,
            self.pause_on_no_internet,
            self.services
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}
