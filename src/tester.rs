use std::{thread, time};
use std::sync::{Arc, Mutex};
use std::process::{Command, Stdio};
use crate::settings::{ResultOutput, Service, Settings, TestResult};
use serde_json::Value;
use process_alive::Pid;
use libc;


pub struct Tester {
    settings: Arc<Mutex<Settings>>
}

impl Tester {
    pub fn new(settings : Arc<Mutex<Settings>>) -> Self {
        Tester {
            settings
        }
    }

    pub fn test(&self){
        let mut settings = self.settings.lock().unwrap();
        let timeout = settings.timeout;
        for test in &mut settings.services {
            let mut command = Command::new(test.command.clone());
            if let Some(args) = &test.args {
                command.args(args);
            }

            let option_output = match command.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn() {
                Ok(child) => {
                    let id = child.id();
                    thread::spawn(move || Tester::suicide_watch(id, timeout));
                    println!("Started test with PID:\t{}\t({})", id, test.name);
                    child.wait_with_output()
                },
                Err(e) => Err(e),
            };

            if option_output.is_err() {
                let result : ResultOutput = ResultOutput::String(option_output.expect_err("Not an error?").to_string());
                test.successes = 0.0;
                test.result = result;
                continue;
            }

            let output = option_output.unwrap();
            let status = output.status;
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            // Command returned a non-zero code
            if !status.success() {
                let result : ResultOutput = if !stderr.is_empty() {
                    ResultOutput::String(stderr.to_string())
                }else if !stdout.is_empty(){
                    ResultOutput::String(stdout.to_string())
                }else if status.code().is_some() {
                    ResultOutput::String(format!("Exited with non-zero code: {}", status.code().unwrap()))
                } else {
                    ResultOutput::String(status.to_string())
                };
                test.successes = 0.0;
                test.result = result;
                continue;
            }


            // We want to support 2 different formats. Here we go
            test.result = match serde_json::from_str::<Value>(&stdout) {
                // JSON
                Ok(value) => self.format_json(value, test.clone()),
                // PLAIN
                Err(_) => self.format_plain(&stdout)
            };

            test.successes = match &test.result {
                ResultOutput::String(_) => 1.0,
                ResultOutput::Bool(b) => b.clone() as i32 as f64,
                ResultOutput::Int(i) => i.clone() as f64,
                ResultOutput::Float(f) => f.clone() as f64,
                ResultOutput::Result(v) => v.iter().map(|val| val.success).sum::<f64>() / v.len() as f64
            }
        }
    }


    fn format_json(&self, value : Value, test : Service) -> ResultOutput { // JSON
        // Must include name (string), success(bool|float), result(string|number|bool|json)
        // Root JSON can be an array or an object
        if !value.is_object() && !value.is_array() {
            panic!("JSON is neither object nor array");
        }
        let tests : Vec<Value> = if value.is_object() {
            vec![Value::Object(value.as_object().unwrap().clone())]
        } else {
            value.as_array().unwrap().to_vec()
        };
        let mut results : Vec<TestResult> = vec![];
        for obj in tests {
            let name = if let Some(name_value) = obj.get("name") {
                if let Some(name_str) = name_value.as_str() {
                    name_str.to_string()
                } else {
                    println!("Invalid format in test object {}; Name must be a string, skipping", test.name);
                    continue;
                }
            } else {
                println!("Invalid format in test object {}; JSON test require a \"name\" key, skipping", test.name);
                continue;
            };

            let success: f64 = match obj.get("success") {
                Some(v) if v.is_f64() => v.as_f64().unwrap(),
                Some(v) if v.is_number() => v.as_i64().unwrap() as f64,
                Some(v) if v.is_boolean() => v.as_bool().unwrap() as i32 as f64,
                _ => {println!("Invalid format in test object {} -> {}; JSON tests require a \"success\" key with a number value (between 0.00 and 1.00), skipping", test.name, name); continue;}
            };
            if success > 1.00 || 0.00 > success {
                println!("Invalid format in test object {} -> {}; JSON tests require the \"success\" key to be a number between 0.00 and 1.00 (or a boolean), skipping", test.name, name);
                continue;
            }

            let result_output = if let Some(result_value) = obj.get("result") {
                result_value
            } else {
                println!("Invalid format in test object {} -> {}; JSON test require a \"result\" key", test.name, name);
                continue;
            };


            results.push(TestResult{
                name,
                success,
                result: result_output.clone()
            });
        }
        ResultOutput::Result(results)
    }

    fn format_plain(&self, value : &str) -> ResultOutput {
        let mut results : Vec<TestResult> = vec![];
        // Validate format
        let mut found_name = false;
        let mut name : String = String::from("");
        let mut success : f64 = -1.0;
        let mut result_builder : String = String::from("");

        fn make_result(name : String, success : f64, result_builder : String) -> TestResult {
            // We want to support 2 different formats. Here we go
            let result = match serde_json::from_str::<Value>(&result_builder) {
                // JSON
                Ok(value) => value,
                // PLAIN
                Err(_) => match serde_json::to_value(result_builder.as_str()) {
                    Ok(value) => value,
                    Err(_) => panic!("WTF!")
                }
            };

            TestResult {
                name,
                success,
                result
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
        }else{
            ResultOutput::String(value.to_string())
        }
    }

    fn suicide_watch(pid_num : u32, timeout : f64){
        thread::sleep(time::Duration::from_secs_f64(timeout));

        // Check if exists
        let pid = Pid::from(pid_num);
        if process_alive::state(pid).is_alive() { // Termination
            unsafe {
                libc::kill(pid_num as i32, 15);
                println!("Process timeout {}s, terminating {}", timeout, pid_num);
            }
        }

        thread::sleep(time::Duration::from_secs_f64(timeout*3.0));
        if process_alive::state(pid).is_alive() { // DIE DIE DIE
            unsafe {
                libc::kill(pid_num as i32, 9);
                println!("Failed to terminate. Force killing process {}", pid_num);
            }
        }
    }
}
