use crate::settings::{ResultOutput, Service, TestResult};
use libc;
use process_alive::Pid;
use serde_json::{json, Value};
use std::process::{Command, Stdio};
use std::{thread, time};

pub struct Tester {}

impl Tester {
    /// Tests a service and returns the success rate and the result of the test.
    ///
    /// # Arguments
    ///
    /// * `service` - A `Service` instance that represents the service to be tested.
    ///
    /// # Returns
    ///
    /// A tuple where the first element is the success rate as a float, and the second element is the result of the test as a `ResultOutput`.
    pub fn test(service: Service) -> (f64, ResultOutput) {
        let mut command = Command::new(service.command.clone());
        if let Some(args) = &service.args {
            command.args(args);
        }

        let option_output = match command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(child) => {
                let id = child.id();
                thread::spawn(move || Tester::suicide_watch(id, service.timeout));
                println!("  {}pid {}", id, service.name);
                child.wait_with_output()
            }
            Err(e) => Err(e),
        };

        if option_output.is_err() {
            println!("? {:.2} {} (Not an error?)", 0.00, service.name);
            return (
                0.0,
                ResultOutput::String(option_output.expect_err("Not an error?").to_string()),
            );
        }

        let output = option_output.unwrap();
        let status = output.status;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Command returned a non-zero code
        if !status.success() {
            let result: ResultOutput = if !stderr.is_empty() {
                ResultOutput::String(stderr.to_string())
            } else if !stdout.is_empty() {
                ResultOutput::String(stdout.to_string())
            } else if status.code().is_some() {
                ResultOutput::String(format!(
                    "Exited with non-zero code: {}",
                    status.code().unwrap()
                ))
            } else {
                ResultOutput::String(status.to_string())
            };
            println!(
                "X {:.2} {} (return code {})",
                0.00,
                service.name,
                status.code().unwrap_or(2522)
            );
            return (0.0, result);
        }

        // We want to support 2 different formats. Here we go
        let result = match serde_json::from_str::<Value>(&stdout) {
            // JSON
            Ok(value) => Tester::format_json(value, service.clone()),
            // PLAIN
            Err(_) => Tester::format_plain(&stdout),
        };

        let successes = match &result {
            ResultOutput::Null => 1.0,
            ResultOutput::String(_) => 1.0,
            ResultOutput::Bool(b) => *b as i32 as f64,
            ResultOutput::Int(i) => *i as f64,
            ResultOutput::Float(f) => *f as f64,
            ResultOutput::Result(v) => {
                v.iter().map(|val| val.success).sum::<f64>() / v.len() as f64
            }
        };

        if successes >= 1.0 {
            // No point printing anything really
            // println!("✓ {:.2} {}", successes, service.name);
        } else if successes > 0.5 {
            println!("↑ {:.2} {}", successes, service.name);
        } else {
            println!("↓ {:.2} {}", successes, service.name);
        }

        return (successes, result);
    }

    /// Formats a JSON value into a `ResultOutput`.
    ///
    /// This function expects the JSON value to be either an object or an array. Each object should have the following keys:
    /// * "name" (string): the name of the test.
    /// * "success" (number): the success rate of the test. It should be a number between 0.00 and 1.00.
    /// * "result" (any): the result of the test.
    ///
    /// # Arguments
    ///
    /// * `value` - A `Value` that represents the JSON value to be formatted.
    /// * `test` - A `Service` instance that represents the service being tested.
    ///
    /// # Returns
    ///
    /// A `ResultOutput` that represents the formatted result.
    ///
    /// # Panics
    ///
    /// This function will panic if the JSON value is neither an object nor an array.
    fn format_json(value: Value, test: Service) -> ResultOutput {
        // JSON
        // Must include name (string), success(bool|float), result(string|number|bool|json)
        // Root JSON can be an array or an object
        if !value.is_object() && !value.is_array() {
            panic!("JSON is neither object nor array");
        }
        let tests: Vec<Value> = if value.is_object() {
            let obj = value.as_object().unwrap();
            // Check if key array
            if obj.iter().all(|(_, v)| v.is_object()) {
                let mut vec = Vec::new();
                for (k, v) in obj {
                    let mut new_obj = v.clone();
                    new_obj["name"] = json!(k);
                    vec.push(new_obj);
                }
                vec
            } else {
                vec![Value::Object(value.as_object().unwrap().clone())]
            }
        } else {
            value.as_array().unwrap().to_vec()
        };
        let mut results: Vec<TestResult> = vec![];
        for obj in tests {
            let name = if let Some(name_value) = obj.get("name") {
                if let Some(name_str) = name_value.as_str() {
                    name_str.to_string()
                } else {
                    println!(
                        "Invalid format in test object {}; Name must be a string, skipping",
                        test.name
                    );
                    continue;
                }
            } else {
                println!(
                    "Invalid format in test object {}; JSON test require a \"name\" key, skipping",
                    test.name
                );
                continue;
            };

            let success: f64 = match obj.get("success") {
                Some(v) if v.is_f64() => v.as_f64().unwrap(),
                Some(v) if v.is_number() => v.as_i64().unwrap() as f64,
                Some(v) if v.is_boolean() => v.as_bool().unwrap() as i32 as f64,
                _ => {
                    println!("Invalid format in test object {} -> {}; JSON tests require a \"success\" key with a number value (between 0.00 and 1.00), skipping", test.name, name);
                    continue;
                }
            };
            if success > 1.00 || 0.00 > success {
                println!("Invalid format in test object {} -> {}; JSON tests require the \"success\" key to be a number between 0.00 and 1.00 (or a boolean), skipping", test.name, name);
                continue;
            }

            let result_output = if let Some(result_value) = obj.get("result") {
                result_value
            } else {
                println!(
                    "Invalid format in test object {} -> {}; JSON test require a \"result\" key",
                    test.name, name
                );
                continue;
            };

            results.push(TestResult {
                name,
                success,
                result: result_output.clone(),
            });
        }

        if !results.is_empty() {
            ResultOutput::Result(results)
        } else {
            ResultOutput::Bool(false)
        }
    }

    /// Formats a plain text value into a `ResultOutput`.
    ///
    /// This function expects the plain text value to be in the following format:
    /// * The first line should be the name of the test.
    /// * The second line should be the success rate of the test. It should be a bool, or a number (int or float) between 0.00 and 1.00.
    /// * The remaining lines should be the description of the test.
    ///
    /// Each test should be separated by an empty line.
    ///
    /// # Arguments
    ///
    /// * `value` - A string that represents the plain text value to be formatted.
    ///
    /// # Returns
    ///
    /// A `ResultOutput` that represents the formatted result.
    ///
    /// # Panics
    ///
    /// This function will panic if it fails to parse the description of the test as a string or a JSON value.
    fn format_plain(value: &str) -> ResultOutput {
        let mut results: Vec<TestResult> = vec![];
        // Validate format
        let mut found_name = false;
        let mut name: String = String::from("");
        let mut success: f64 = -1.0;
        let mut result_builder: String = String::from("");

        fn make_result(name: String, success: f64, result_builder: String) -> TestResult {
            // We want to support 2 different formats. Here we go
            let result = match serde_json::from_str::<Value>(&result_builder) {
                // JSON
                Ok(value) => value,
                // PLAIN
                Err(_) => match serde_json::to_value(result_builder.as_str()) {
                    Ok(value) => value,
                    Err(_) => panic!("WTF!"),
                },
            };

            TestResult {
                name,
                success,
                result,
            }
        }

        for line in value.lines() {
            if line.is_empty() && success != -1.0 {
                results.push(make_result(name, success, result_builder));
                found_name = false;
                name = String::from("");
                result_builder = String::from("");
                success = -1.0;
                continue;
            }
            if found_name == false {
                // Name
                name = line.to_string();
                found_name = true;
                continue;
            }

            // Success
            if line.to_string().parse::<f64>().is_ok() {
                success = line.to_string().parse::<f64>().unwrap();
                continue;
            }
            if line.to_string().parse::<i32>().is_ok() {
                success = line.to_string().parse::<i32>().unwrap() as f64;
                continue;
            }
            if line.to_string().parse::<bool>().is_ok() {
                success = line.to_string().parse::<bool>().unwrap() as i32 as f64;
                continue;
            }

            // Result
            result_builder += line;
        }

        if success != -1.0 {
            results.push(make_result(name, success, result_builder));
        }

        if !results.is_empty() {
            ResultOutput::Result(results)
        } else if !value.is_empty() {
            ResultOutput::String(value.to_string())
        } else {
            ResultOutput::Null
        }
    }

    /// Monitors a process and terminates it if it exceeds a specified timeout.
    ///
    /// This function will first put the current thread to sleep for the duration of the timeout.
    /// After waking up, it will check if the process is still alive.
    /// If it is, it will attempt to terminate the process.
    /// If the process is still alive after another delay (three times the original timeout), it will forcefully kill the process.
    ///
    /// # Arguments
    ///
    /// * `pid_num` - The PID of the process to be monitored.
    /// * `timeout` - The duration (in seconds) to wait before terminating the process.
    ///
    /// # Safety
    ///
    /// This function is `unsafe` because it calls `libc::kill`, which can lead to undefined behavior if not used correctly.
    fn suicide_watch(pid_num: u32, timeout: f64) {
        thread::sleep(time::Duration::from_secs_f64(timeout));

        // Check if exists
        let pid = Pid::from(pid_num);
        if process_alive::state(pid).is_alive() {
            // Termination
            unsafe {
                libc::kill(pid_num as i32, 15);
                println!("Process timeout {}s, terminating {}", timeout, pid_num);
            }
        }

        thread::sleep(time::Duration::from_secs_f64(timeout * 3.0));
        if process_alive::state(pid).is_alive() {
            // DIE DIE DIE
            unsafe {
                libc::kill(pid_num as i32, 9);
                println!("Failed to terminate. Force killing process {}", pid_num);
            }
        }
    }
}
