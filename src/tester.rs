use std::sync::{Arc, Mutex};
use std::process::{Command, ExitStatus};
use serde::{Deserialize, Serialize};
use crate::settings::{BoolOrFloat, ResultOutput, Settings, TestResult};
use serde_json::{to_string, Value};


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
        for test in &mut settings.services {
            let mut results : Vec<ResultOutput> = vec![];
            let optionOutput = Command::new(test.command.clone())
                .output();

            if optionOutput.is_err() {
                let result : ResultOutput = ResultOutput::String(optionOutput.expect_err("Not an error?").to_string());
                results.push(result);
                test.successes = 0.0;
                test.result = results;
                continue;
            }

            let output = optionOutput.unwrap();
            let status = output.status;
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            // Command returned a non-zero code
            if !status.success() {
                let result : ResultOutput = if stderr.is_empty() {
                    ResultOutput::String(format!("Exited with non-zero code: {}", status.code().unwrap()))
                }else{
                    ResultOutput::String(stderr.to_string())
                };
                results.push(result);
                test.successes = 0.0;
                test.result = results;
                continue;
            }

            // We want to support 2 different formats. Here we go
            match serde_json::from_str::<Value>(&stdout) {
                Ok(value) => { // JSON
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

                        let success: BoolOrFloat = match obj.get("success") {
                            Some(v) if v.is_f64() => BoolOrFloat::Float(v.as_f64().unwrap()),
                            Some(v) if v.is_number() => BoolOrFloat::Float(v.as_i64().unwrap() as f64),
                            Some(v) if v.is_boolean() => BoolOrFloat::Bool(v.as_bool().unwrap()),
                            _ => {println!("Invalid format in test object {} -> {}; JSON tests require a \"success\" key with a boolean or number value, skipping", test.name, name); continue;}
                        };

                        let result_output = if let Some(result_value) = obj.get("result") {
                            result_value
                        } else {
                            println!("Invalid format in test object {} -> {}; JSON test require a \"result\" key", test.name, name);
                            continue;
                        };


                        results.push(ResultOutput::Result(TestResult{
                            name,
                            success,
                            result: result_output.clone()
                        }));
                    }
                    test.result = results;
                }
                Err(e) => {
                    println!("しまった！");
                }
            }
        }

        // println!("{:#?}", settings);
    }
}
