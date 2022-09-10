mod crawler;
mod storage;

use std::cell::RefCell;
use std::env::current_dir;
use std::path::PathBuf;
use std::rc::Rc;
use std::thread::sleep;
use std::time::Duration;

use crawler::ChromeDriver;

/// Type aliasing for Box<dyn std::error::Error> that is used globally.
pub type Exception = Box<dyn std::error::Error>;

/// The entry point of the app.
pub fn run_app() -> Result<(), Exception> {
    // Initialize the crawler and the flag as a mutable reference.
    let web_driver = ChromeDriver::new()?;
    let crawler = Rc::new(RefCell::new(web_driver));
    let flag = Rc::new(RefCell::new(false));

    loop {
        let mut crawler_mut = crawler.borrow_mut();
        match crawler_mut.is_now() {
            Ok(bool_value) => { if bool_value {
                // Set the event off only when
                // "bool_value" == true && "flag" == false.
                let mut flag_mut = flag.borrow_mut();
                if !(*flag_mut) {
                    match crawler_mut.search() {
                        Ok(()) => {},
                        Err(e) => { dbg!(e); }
                    }
                    *flag_mut = true;
                    continue
                } else {
                    sleep(Duration::from_secs(1));
                    continue
                }
            } else {
                // Otherwise, set the flag back to false.
                let mut flag_mut = flag.borrow_mut();
                *flag_mut = false;
            }},
            Err(e) => { 
                dbg!(e);
                sleep(Duration::from_secs(1)); 
            },
        }
    }
}

// /// Load configurations from the Settings.toml file located at
// /// the program root directory.
// fn load_config() -> Result<Config, Exception> {
//     // The base path for configs ("./Settings.toml").
//     let mut settings_path = current_dir()?;
//     settings_path.push("Settings.toml");
//     let settings_path_str = settings_path.to_str().unwrap();

//     // Build the config file.
//     let config = Config::builder()
//         .add_source(config::File::with_name(settings_path_str))
//         .add_source(config::Environment::with_prefix("APP"))
//         .build()?;
//     Ok(config)
// }

fn load_csv_path() -> Result<PathBuf, Exception> {
    let mut csv_path = current_dir()?;
    csv_path.push("Papers.csv");
    Ok(csv_path)
}