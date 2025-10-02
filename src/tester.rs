use libc;
use process_alive::Pid;
use serde_json::{json, Value};
use std::process::{Command, Stdio};
use std::{thread, time};
use std::collections::HashMap;
use crate::service::Service;
use crate::settings::{ResultOutput, TestResult};
use crate::utils::retry_strategy::RetryStrategy;

type SuccessResult = (f64, ResultOutput);

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
    pub fn test(service: &Service) -> SuccessResult {
        let mut command = Command::new(service.command.clone());
        if let Some(args) = &service.args {
            command.args(args);
        }
        
        let mut results: Vec<SuccessResult> = vec!();
        
        let retries = service.retry_counter;
        for retry_count in 0..=retries {
            let option_output = match command
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
            {
                Ok(child) => {
                    let id = child.id();
                    let timeout = service.timeout;
                    thread::spawn(move || Tester::suicide_watch(id, timeout));
                    println!("   {}pid {}", id, service.name);
                    child.wait_with_output()
                }
                Err(e) => Err(e),
            };

            if let Err(e) = option_output {
                let err_msg = format!("Internal error: {}", e);
                let success_result = (0.0, ResultOutput::String(err_msg.to_string()));
                if retries > 0 {
                    if retries > retry_count {
                        eprintln!("?⟳ {:.2} {} {}", 0.00, service.name, err_msg);
                    } else {
                        eprintln!("?  {:.2} {} {}", 0.00, service.name, err_msg);
                    }
                    results.push(success_result);
                    continue;
                } else {
                    eprintln!("?  {:.2} {} {}", 0.00, service.name, err_msg);
                    return success_result;
                }
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
                        status.code().unwrap_or(9999)
                    ))
                } else {
                    ResultOutput::String(status.to_string())
                };
                let err_msg = format!("Non-zero exit code: {}", status.code().unwrap_or(2522));
                let success_result = (0.0, result);
                if retries > 0 {
                    if retries > retry_count {
                        eprintln!("X⟳ {:.2} {} {}", 0.00, service.name, err_msg);
                    } else {
                        eprintln!("X  {:.2} {} {}", 0.00, service.name, err_msg);
                    }
                    results.push(success_result);
                    continue;
                } else {
                    eprintln!("X  {:.2} {} {}", 0.00, service.name, err_msg);
                    return success_result;
                }
            }

            // We want to support 2 different formats. Here we go
            let result: ResultOutput = match serde_json::from_str::<Value>(&stdout) {
                // JSON
                Ok(value) => Tester::format_json(value, service),
                // PLAIN
                Err(_) => Tester::format_plain(&stdout),
            };

            let successes = result.to_successes();
            if 1.0 > successes {
                let mut icons = String::new();
                if successes > 0.5 {
                    icons += "↑";
                } else {
                    icons += "↓";
                }
                if retries > retry_count {
                    icons += "⟳";
                } else {
                    icons += " ";
                }
                println!("{} {:.2} {}", icons, successes, service.name);
            }
            
            if retries == 0 {
                // Return early skipping expensive vector push operation
                // Also skipping the entire retry_strategy logic
                return (successes, result);
            } else if successes == 1.0
                && (service.retry_strategy == RetryStrategy::Best
                || service.retry_strategy == RetryStrategy::CombinedBest
                || service.retry_strategy == RetryStrategy::Median
            ) {
                // If we already have a success, and we're looking for the best result(s), just return without continuing
                return (successes, result);
            } else {
                results.push((successes, result));   
            }
        }
        
        Tester::combine_results(results, &service.retry_strategy)
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
    fn format_json(value: Value, test: &Service) -> ResultOutput {
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

    /// Combines multiple `SuccessResult` into a single combined `SuccessResult` based on the `RetryStrategy` strategy set
    ///
    /// Strategy:
    /// - The `retry_strategy` determines which attempt(s) are shown and how the final
    ///   score is computed (e.g., last attempt, best score, worst score, or averaged).
    /// - Ties and empty inputs are handled deterministically.
    ///
    /// Arguments:
    /// - `results`: Multiple results from the same test accumulated by multiple retries.
    /// - `retry_strategy`: Policy that controls how to merge the results.
    /// 
    /// Returns:
    /// - `(score, ResultOutput)`: The aggregated success score and the merged `ResultOutput`.
    fn combine_results(results: Vec<SuccessResult>, retry_strategy: &RetryStrategy) -> SuccessResult {
        match retry_strategy {
            // This just grabs the best result found
            RetryStrategy::Best => {
                let mut best_result = &results[0];
                for result in &results {
                    let (s, _) = result;
                    if s > &best_result.0 {
                        best_result = result;
                    }
                }
                best_result.to_owned()
            },
            // Most relevant when using `ResultOutput::Result(Vec<TestResult>)`.
            // It will combine all the TestResults and find the best test for each vector.
            RetryStrategy::CombinedBest => {
                
                // The combined_best flag does not make sense if all the Vec<TestResult> are empty or not set.
                // Therefore, default back to flag "best" if this is the case.
                if results.iter().all(|(_, r)| match r {
                    ResultOutput::Result(v) => v.is_empty(),
                    _ => true,
                }) {
                    let mut best_result = &results[0];
                    for result in &results {
                        let (s, _) = result;
                        if s > &best_result.0 {
                            best_result = result;
                        }
                    }
                    return best_result.to_owned();
                }
                
                let mut best_results: HashMap<String, TestResult> = HashMap::new();
                for (_s, result) in results {
                    match result {
                        ResultOutput::Result(v) => {
                            for tr in v {
                                let key = tr.name.clone();
                                if let Some(best_result) = best_results.get(&key) {
                                    if tr.success > best_result.success {
                                        best_results.insert(key, tr);
                                    }
                                } else {
                                    best_results.insert(key, tr); 
                                }
                            }
                        },
                        _ => continue,
                    }
                }
                let result: ResultOutput = ResultOutput::Result(best_results.values().cloned().collect());
                let successes = result.to_successes();
                (successes, result)
                
            },
            RetryStrategy::Median => {
                // Orders by success rate and picks the middle result
                // This is generally a bit of a heavy operation to do
                let count = results.len();
                let mut results = results.to_owned();
                results.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
                let mid = count / 2;
                results[mid].to_owned()
            },
            // This just grabs the worst result found
            RetryStrategy::Worst => {
                let mut worst_result = &results[0];
                for result in &results {
                    let (s, _) = result;
                    if &worst_result.0 > s {
                        worst_result = result;
                    }
                }
                worst_result.to_owned()
            },
            // Most relevant when using `ResultOutput::Result(Vec<TestResult>)`.
            // It will combine all the TestResults and find the worst test for each vector, combining them into one.
            RetryStrategy::CombinedWorst => {

                // The combined_best flag does not make sense if all the Vec<TestResult> are empty or not set.
                // Therefore, default back to flag "best" if this is the case.
                if results.iter().all(|(_, r)| match r {
                    ResultOutput::Result(v) => v.is_empty(),
                    _ => true,
                }) {
                    let mut worst_result = &results[0];
                    for result in &results {
                        let (s, _) = result;
                        if &worst_result.0 > s {
                            worst_result = result;
                        }
                    }
                    return worst_result.to_owned();
                }

                // We use a hashmap, and use the test result name as the key. This is unique for each test.
                let mut worst_results: HashMap<String, TestResult> = HashMap::new();
                for (_s, result) in results {
                    match result {
                        ResultOutput::Result(v) => {
                            for tr in v {
                                let key = tr.name.clone();
                                if let Some(worst_result) = worst_results.get(&key) {
                                    if worst_result.success > tr.success {
                                        worst_results.insert(key, tr);
                                    }
                                } else {
                                    worst_results.insert(key, tr);
                                }
                            }
                        },
                        _ => continue,
                    }
                }
                let result: ResultOutput = ResultOutput::Result(worst_results.values().cloned().collect());
                let successes = result.to_successes();
                (successes, result)
            },
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
