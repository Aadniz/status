use std::sync::{Arc, Mutex};
use std::process::{Command, ExitStatus};
use crate::settings::{ResultOutput, Settings};
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

            if (optionOutput.is_err()){
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

            if (!status.success()){
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
        }

        println!("{:#?}", settings);
    }
}
