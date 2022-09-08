use std::ffi::OsString;
use std::sync::Arc;
use std::time::Duration;

use headless_chrome::{Browser, Element, LaunchOptionsBuilder, Tab};
use rayon::prelude::*;

use crate::Exception;
use crate::scheduler::Scheduler;
use crate::storage::{Paper, Storage};

/// # ChromeDriver
/// 
/// Blocking client
pub struct ChromeDriver {
    #[allow(unused)]
    browser: Browser,
    main_tab: Arc<Tab>,
    domain_string: String,
    base_query_string: String,
    blank_token: String,
    max_indices_per_page: usize,
    storage: Storage,
}

impl ChromeDriver {
    /// The function initializes the web driver client with a read-only javascript "Tab" object.
    /// 
    /// [WARNING]
    /// 
    /// Although "Arc<Tab>" seems to be thread-safe, the Tab object is actually a web api call
    /// that returns a shared reference to the current window handle. Javascript Window object
    /// can be mutated at any point without the Rust implementation of interior mutability.
    pub fn new() -> Result<Self, Exception> {
        let user_agent = OsString::from("--user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.0.0 Safari/537.36");
        let options = LaunchOptionsBuilder::default()
            .args(vec![&user_agent])
            .headless(true)
            .build()?;
        let browser = Browser::new(options)?;
        let main_tab = browser.wait_for_initial_tab()?;

        Ok(Self {
            browser,
            main_tab,
            domain_string: "https://www.sciencedirect.com/".into(),
            base_query_string: "https://www.sciencedirect.com/search?qs=".into(),
            blank_token: "%20".into(),
            max_indices_per_page: 50,
            storage: Storage::new(),
        })
    }

    /// Adds a new keyword to search for.
    fn query_from_keyword(&self, keyword: &str) -> Result<String, Exception> {
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
        query.push_str("&sortBy=date");

        Ok(query)
    }

    /// The function starts searching for result for each keyword, 
    /// parses the html element, filters the result and saves changes.
    pub fn search(&mut self, scheduler: &Scheduler) -> Result<(), Exception> {
        let outer_selector = "#srp-results-list";
        let last_element = format!("#srp-results-list > ol > li:nth-child({})", self.max_indices_per_page);

        // Scrape the page with initialized query strings.
        let new_keyword = scheduler.keyword();
        for keyword in new_keyword {
            let url = self.query_from_keyword(keyword)?;
            self.main_tab
                .navigate_to(&url)?
                .wait_until_navigated()?
                .wait_for_element_with_custom_timeout(&last_element, Duration::from_millis(10000))?;
                
            // Timeout set to 10 seconds.
            let result_list = self.main_tab.wait_for_element_with_custom_timeout(
                &outer_selector, 
                Duration::from_millis(10000)
            )?;
            let li_list = result_list.wait_for_elements("li")?;

            // Parallel parse() execution.
            self.parse(li_list, keyword, &self.domain_string, scheduler)?;
        }
        self.storage.update(new_keyword);

        Ok(())
    }

    /// Multi-threaded parser utilizing ["rayon"].
    fn parse(
        &self, 
        item_list: Vec<Element>, 
        keyword: &str, 
        domain: &str,
        scheduler: &Scheduler,
    ) -> Result<(), Exception> {
        let storage = &self.storage;
        let scheduler = scheduler;

        // Parse items in the list.
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

                    // Parse href and uref out of the content string.
                    let href = {
                        let content = elements[0].get_content().unwrap();
                        let tokens: Vec<_> = content.split('"').collect();

                        // The complete href.
                        let mut href = String::from(domain);
                        href.push_str(tokens[3]);

                        href
                    };

                    // Build the paper struct.
                    let paper = Paper {
                        title: elements[0].get_inner_text().unwrap(),
                        href: href.to_string(),
                        keyword: keyword.into(),
                        journal: elements[1].get_inner_text().unwrap(),
                    };

                    // Build the uid tuple
                    let uid = (keyword.to_string(), href.to_string());
                    let result = storage.insert(uid, paper.clone());
                    
                    // Write to the file.
                    if result {
                        scheduler.write(paper).unwrap();
                    }
                }
            });

        Ok(())
    }
}