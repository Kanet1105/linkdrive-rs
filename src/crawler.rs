use std::sync::Arc;
use std::time::Duration;

use super::Exception;
use super::storage::Paper;

use headless_chrome::{Browser, Element, LaunchOptionsBuilder, Tab};
use rayon::prelude::*;

/// # ChromeDriver
/// 
/// Blocking client
pub struct ChromeDriver {
    browser: Browser,
    main_tab: Arc<Tab>,
    domain_string: String,
    base_query_string: String,
    blank_token: String,
    query_string: Vec<(String, String)>,
    max_indices_per_page: usize,
}

impl ChromeDriver {
    /// The function initializes the web driver client with a read-only javascript Tab object.
    /// 
    /// [WARNING]
    /// 
    /// Although Arc<Tab> seems to be thread-safe, the Tab object is actually a web api call
    /// that returns a shared reference to the current window handle. Javascript Window object
    /// can be mutated at any point without the Rust implementation of interior mutability.
    pub fn new() -> Result<Self, Exception> {
        let options = LaunchOptionsBuilder::default()
        .headless(false)
        .build()?;
        let browser = Browser::new(options)?;
        let tab = browser.wait_for_initial_tab()?;

        Ok(Self {
            browser,
            main_tab: tab,
            domain_string: "https://www.sciencedirect.com/".into(),
            base_query_string: "https://www.sciencedirect.com/search?qs=".into(),
            blank_token: "%20".into(),
            query_string: Vec::<(String, String)>::new(),
            max_indices_per_page: 25,
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
    /// 
    ///     Ok(())
    /// }
    /// ```
    pub fn add_keyword(&mut self, keyword: &str) -> Result<(), Exception> {
        // Split keyword argument at whitespaces into a token vector.
        let token = keyword
            .split_ascii_whitespace()
            .into_iter()
            .map(|x| String::from(x))
            .collect::<Vec<String>>();

        // Join tokens with "self.blank_token" separator.
        let search_keyword = token.join(&self.blank_token);

        // Build a query string from joining "self.base_query_string" and
        // the search keyword.
        let mut query = String::from(&self.base_query_string);
        query.push_str(&search_keyword);
        query.push_str(&format!("&show={}", self.max_indices_per_page));
        self.query_string.push((query, keyword.into()));

        Ok(())
    }

    /// The function starts searching for result for each keyword, parse the element, 
    /// filter the result and saves only changes.
    pub fn search(&self) -> Result<(), Exception> {
        let outer_selector = "#srp-results-list";
        let last_element = format!("#srp-results-list > ol > li:nth-child({})", self.max_indices_per_page);

        // Scrape the page with initialized query strings.
        for (url, keyword) in &self.query_string {
            self.main_tab
                .navigate_to(url)?
                .wait_until_navigated()?
                .wait_for_element_with_custom_timeout(&last_element, Duration::from_millis(10000))?;
                
            // Timeout set to 10 seconds.
            let result_list = self.main_tab.wait_for_element_with_custom_timeout(&outer_selector, Duration::from_millis(10000))?;
            let item_list = result_list.wait_for_elements("li")?;

            // Parallel parse() execution.
            self.parse(item_list, keyword, &self.domain_string)?;
        }

        Ok(())
    }

    /// Multi-threaded parser utilizing ["rayon"].
    pub fn parse(&self, item_list: Vec<Element>, keyword: &str, domain: &str) -> Result<(), Exception> {
        item_list
            .par_iter()
            .for_each(|item| {
                // Get attributes to check if the html element contains a valid result.
                let attr = item.get_attributes()
                    .unwrap()
                    .unwrap();

                // Continue when "!attr.is_empty() and exclude the download link."
                if !attr.is_empty() && attr.len() == 4 {
                    let elements = item.wait_for_elements("a").unwrap();

                    // Parse href and uid out of the content string.
                    let (href, uid) = {
                        let content = elements[0].get_content().unwrap();
                        let tokens: Vec<_> = content.split('"').collect();

                        // The complete href.
                        let mut href = String::from(domain);
                        href.push_str(tokens[3]);
                        (href, tokens[7].to_string())
                    };

                    // Build the paper struct.
                    let paper = Paper {
                        title: elements[0].get_inner_text().unwrap(),
                        href,
                        keyword: keyword.into(),
                        journal: elements[1].get_inner_text().unwrap(),
                        date_published: "".to_string(),
                    };

                    println!("{:?}", &paper);
                    println!("======================================================");    
                }
            });

        Ok(())
    }
}