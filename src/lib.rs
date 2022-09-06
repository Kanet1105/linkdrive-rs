mod crawler;
mod storage;

use std::collections::HashMap;
use std::env::current_dir;

use crawler::ChromeDriver;

use config::{self, Config, Value};

/// Type aliasing for Box<dyn std::error::Error> that is used globally.
pub type Exception = Box<dyn std::error::Error>;

/// The entry point of the app.
pub fn run_app() -> Result<(), Exception> {
    let web_driver = ChromeDriver::new()?;

    loop {
        let config = load_config()?.get_table("default")?;
        let keyword = extract(&config, "keyword")?;
        web_driver.search(&keyword)?;

        std::thread::sleep(std::time::Duration::from_secs(10));
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

/// Extract the vector from the config hashmap and cast "Value" to "String".
fn extract(config: &HashMap<String, Value>, key: &str) -> Result<Vec<String>, Exception> {
    let extracted: Vec<String> = config
        .get(key).unwrap()
        .clone()
        .into_array()?
        .iter()
        .map(|x| { x.to_string() })
        .collect();
    Ok(extracted)
}