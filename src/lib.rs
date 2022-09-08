mod crawler;
mod scheduler;
mod storage;

use std::env::current_dir;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;

use chrono::prelude::*;
use config::{self, Config};

use crawler::ChromeDriver;

/// Type aliasing for Box<dyn std::error::Error> that is used globally.
pub type Exception = Box<dyn std::error::Error>;

/// The entry point of the app.
pub fn run_app() -> Result<(), Exception> {
    let mut flag = true;
    // Initialize the crawler as a mutable reference.
    let mut crawler = ChromeDriver::new()?;

    loop {
        crawler.search()?;
        sleep(Duration::from_secs(2));
    }
}

/// Load configurations from the Settings.toml file located at
/// the program root directory.
fn load_config() -> Result<Config, Exception> {
    // The base path for configs ("./Settings.toml").
    let mut settings_path = current_dir()?;
    settings_path.push("Settings.toml");
    let settings_path_str = settings_path.to_str().unwrap();

    // Build the config file.
    let config = Config::builder()
        .add_source(config::File::with_name(settings_path_str))
        .build()?;

    Ok(config)
}

fn load_csv_path() -> Result<PathBuf, Exception> {
    let mut csv_path = current_dir()?;
    csv_path.push("Papers.csv");

    Ok(csv_path)
}

/// Set the alarm off.
fn is_now(alarm_time: (u32, u32, Weekday)) -> bool {
    let local = Local::now();
    let (h, m, w) = alarm_time;
    if local.weekday() == w && local.hour() == h && local.minute() == m {
        true
    } else {
        false
    }
}