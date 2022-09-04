mod crawler;
mod storage;

use crawler::ChromeDriver;

/// Type aliasing for Box<dyn std::error::Error> that is used globally.
pub type Exception = Box<dyn std::error::Error>;

/// The entry point of the app.
pub fn run_app() -> Result<(), Exception> {
    let mut web_driver = ChromeDriver::new()?;
    web_driver.add_keyword("ai")?;
    web_driver.add_keyword("supply chain")?;
    web_driver.search()?;
    web_driver.search()?;
    
    Ok(())
}

#[test]
fn csv_writer() {
    
    // let mut writer = 
}