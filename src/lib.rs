mod crawler;
mod perf;
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

    Ok(())
}

#[test]
pub fn test() -> Result<(), Exception> {
    use std::time::Duration;
    use headless_chrome::{Browser, LaunchOptionsBuilder};

    let options = LaunchOptionsBuilder::default()
        .headless(false)
        .build()?;
    let browser = Browser::new(options)?;
    let tab = browser.wait_for_initial_tab()?;
    tab.navigate_to("https://www.sciencedirect.com/search?qs=ai&show=100")?;
    tab.wait_until_navigated()?;
    
    let outer_selector = "#main_content > div.SearchBody.row.transparent > div.transparent.results-container.col-xs-24.col-sm-16.col-lg-18.hidden-checkboxes.visible";
    let result_list = tab.wait_for_element_with_custom_timeout(&outer_selector, Duration::from_millis(5000))?;
    let a_list = result_list.wait_for_elements("a")?;
    Ok(())
}