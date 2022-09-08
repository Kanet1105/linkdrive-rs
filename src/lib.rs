mod crawler;
mod scheduler;
mod storage;

use std::env::current_dir;
use std::path::PathBuf;

use config::{self, Config};

use crawler::ChromeDriver;
use scheduler::Scheduler;

/// Type aliasing for Box<dyn std::error::Error> that is used globally.
pub type Exception = Box<dyn std::error::Error>;

/// The entry point of the app.
pub fn run_app() -> Result<(), Exception> {
    // Initialize the default scheduler as a mutable reference.
    let mut scheduler = Scheduler::default();

    // Initialize the Chrome web driver as a mutable reference.
    let mut web_driver = ChromeDriver::new()?;

    // Apply any changes made to "Settings.toml" in a loop.
    loop {
        // Update the scheduler and search.
        match scheduler.update_scheduler() {
            Err(e) => {
                dbg!(e);
                std::thread::sleep(std::time::Duration::from_secs(2));
                continue
            },
            Ok(_) => {
                functional(&mut web_driver, &mut scheduler)?;
                std::thread::sleep(std::time::Duration::from_secs(2));
                continue
            },
        }
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

fn functional(
    web_driver: &mut ChromeDriver, 
    scheduler: &mut Scheduler,
) -> Result<(), Exception> {
    if !scheduler.get_did_search() {
        web_driver.search(scheduler)?;
        scheduler.set_did_search();
    }

    if scheduler.is_now() {
        scheduler.send_email()?;
        scheduler.new_buffer()?;
    }

    Ok(())
}