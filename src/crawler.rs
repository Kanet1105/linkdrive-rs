use std::sync::Arc;
use std::time::Duration;

use headless_chrome::{Browser, LaunchOptionsBuilder, Tab};

/// # ChromeDriver
/// 
/// Blocking client
pub struct ChromeDriver {
    browser: Browser,
    main_tab: Arc<Tab>,
    base_query_string: String,
    blank_token: String,
    query_string: Vec<String>,
    max_indices_per_page: u8,
}

impl ChromeDriver {
    /// The function initializes the web driver client with a read-only javascript Tab object.
    /// 
    /// [WARNING]
    /// 
    /// Although Arc<Tab> seems to be thread-safe, the Tab object is actually a web api call
    /// that returns a shared reference to the current window handle. Javascript Window object
    /// can be mutated at any point without the Rust implementation of interior mutability.
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let options = LaunchOptionsBuilder::default()
        .headless(false)
        .build()?;
        let browser = Browser::new(options)?;
        let tab = browser.wait_for_initial_tab()?;

        Ok(Self {
            browser,
            main_tab: tab,
            base_query_string: "https://www.sciencedirect.com/search?qs=".into(),
            blank_token: "%20".into(),
            query_string: Vec::<String>::new(),
            max_indices_per_page: 100,
        })
    }

    /// Adds a new keyword to search for.
    /// 
    /// [Example]
    /// ```
    /// pub fn run_app() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut web_driver = ChromeDriver::new()?;
    ///     web_driver.add_keyword("ai")?;
    ///     web_driver.add_keyword("supply chain")?;
    ///     web_driver.search()?;
    ///     Ok(())
    /// }
    /// ```
    pub fn add_keyword(&mut self, keyword: &str) -> Result<(), Box<dyn std::error::Error>> {
        let token = keyword
            .split_ascii_whitespace()
            .into_iter()
            .map(|x| String::from(x))
            .collect::<Vec<String>>();

        let search_keyword = token.join(&self.blank_token);
        let mut query = String::from(&self.base_query_string);
        query.push_str(&search_keyword);
        query.push_str(&format!("&show={}", self.max_indices_per_page));

        self.query_string.push(query);
        Ok(())
    }

    pub fn search(&self) -> Result<(), Box<dyn std::error::Error>> {
        // too long..
        let outer_selector = "#main_content > div.SearchBody.row.transparent > div.transparent.results-container.col-xs-24.col-sm-16.col-lg-18.hidden-checkboxes.visible";

        for url in &self.query_string {
            self.main_tab
                .navigate_to(url)?
                .wait_until_navigated()?;
                
            // timeout up to 10 seconds.
            let result_list = self.main_tab.wait_for_element_with_custom_timeout(&outer_selector, Duration::from_millis(10000))?;
            let a_list = result_list.wait_for_elements("a")?;
            // for a in a_list {
            //     println!("{}", a.get_content()?);
            //     println!("====================================================");
            // }
        }
        Ok(())
    }
}