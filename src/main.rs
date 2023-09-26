use std::{thread, time};
use std::sync::{Arc, Mutex};
use crate::pipes::PipeHandler;
use crate::settings::Settings;
use crate::tester::Tester;


// headers
pub mod settings;
pub mod tester;
pub mod pipes;


fn main()
{
    let settings = Arc::new(Mutex::new(Settings::new(None)));
    let tester = Tester::new(Arc::clone(&settings));
    let mut pipe = PipeHandler::new(Arc::clone(&settings));

    thread::spawn(move || pipe.listen());

    loop {
        tester.test();

        let check_interval : u64 = {
            // Creating its own scope to prevent holding onto settings
            let settings = settings.lock().unwrap();
            settings.check_interval
        };
        thread::sleep(time::Duration::from_millis(check_interval * 1000));
    }
}


