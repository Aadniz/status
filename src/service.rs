use std::fmt;
use chrono::prelude::*;
use chrono::serde::ts_seconds_option;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::settings::{ResultOutput, Settings};

/// The `Service` struct represents a service that can be tested.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Service {
    pub name: String,
    pub command: String,
    pub args: Option<Vec<String>>,
    pub interval: u64,
    pub timeout: f64,
    #[serde(with = "ts_seconds_option")]
    pub last_run: Option<DateTime<Utc>>,
    pub successes: f64,
    pub pause_on_no_internet: bool,
    pub result: ResultOutput,
}
impl Service {
    /// Creates a new `Service` instance.
    ///
    /// # Arguments
    ///
    /// * `value` - A reference to a `Value` that contains the service settings.
    /// * `settings` - A `Settings` instance that contains the global settings in case it isn't defined in the service settings.
    ///
    /// # Returns
    ///
    /// A new `Service` instance.
    pub fn new(value: &Value, settings: Settings) -> Self {
        let name = value
            .get("name")
            .expect("Missing name value in service")
            .as_str()
            .expect("Name is not a valid string");
        let command = value
            .get("command")
            .expect("Missing command value in service")
            .as_str()
            .expect("Command is not a valid string");
        let args: Option<Vec<String>> = value.get("args").map(|v| {
            v.as_array()
                .expect("args is not a valid array")
                .iter()
                .map(|s| s.as_str().expect("arg is not a valid string").to_string())
                .collect()
        });
        let interval = value
            .get("interval")
            .and_then(|v| v.as_u64())
            .unwrap_or(settings.interval);
        let timeout = value
            .get("timeout")
            .and_then(|v| v.as_f64())
            .unwrap_or(settings.timeout);
        let pause_on_no_internet = value
            .get("pause_on_no_internet")
            .and_then(|v| v.as_bool())
            .unwrap_or(settings.pause_on_no_internet);

        Service {
            name: String::from(name),
            command: String::from(command),
            args,
            interval,
            timeout,
            last_run: None,
            pause_on_no_internet,
            successes: 0.00,
            result: ResultOutput::Bool(false),
        }
    }
}

impl fmt::Display for Service {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "name: {}, command: {}, interval: {}, timeout: {}",
            self.name, self.command, self.interval, self.timeout
        )
    }
}