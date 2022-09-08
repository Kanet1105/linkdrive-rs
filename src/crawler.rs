use std::fs::File;
use std::ffi::OsString;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use chrono::prelude::*;
use csv::Writer;

use headless_chrome::{Browser, Element, LaunchOptionsBuilder, Tab};
use rayon::prelude::*;

use crate::load_csv_path;
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
    settings: Scheduler,
    file_handle: RwLock<Writer<File>>,
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
            .headless(false)
            .build()?;
        let browser = Browser::new(options)?;
        let main_tab = browser.wait_for_initial_tab()?;
        let file_handle = Writer::from_path(load_csv_path()?)?;

        Ok(Self {
            browser,
            main_tab,
            domain_string: "https://www.sciencedirect.com/".into(),
            base_query_string: "https://www.sciencedirect.com/search?qs=".into(),
            blank_token: "%20".into(),
            max_indices_per_page: 50,
            storage: Storage::new(),
            settings: Scheduler::default(),
            file_handle: RwLock::new(file_handle),
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
    pub fn search(&mut self) -> Result<(), Exception> {
        // Apply changes in "Settings.toml" file to our search.
        let alarm_time = self.settings.update_scheduler()?;
        if !self.is_now(alarm_time) {
            return Ok(())
        }

        let outer_selector = "#srp-results-list";
        let last_element = format!("#srp-results-list > ol > li:nth-child({})", self.max_indices_per_page);

        // Scrape the page with initialized query strings.
        let new_keyword = self.settings.keyword();
        for keyword in new_keyword {
            let url = self.query_from_keyword(keyword)?;
            dbg!(&url);
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
            self.parse(li_list, keyword, &self.domain_string)?;
        }
        self.storage.update(new_keyword);

        // Send an email.
        let local_time = Local::now()
            .naive_local()
            .to_string();
        self.settings.send_email(&local_time)?;
        
        // Get a new file handle.
        self.new_file_handle()?;

        Ok(())
    }

    /// Multi-threaded parser utilizing ["rayon"].
    fn parse(
        &self, 
        item_list: Vec<Element>, 
        keyword: &str, 
        domain: &str,
    ) -> Result<(), Exception> {
        let storage = &self.storage;
        let buffer = &self.file_handle;

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
                        let mut writer = buffer.write().unwrap();
                        writer.serialize(paper).unwrap();
                        writer.flush().unwrap();
                    }
                }
            });

        Ok(())
    }

    fn new_file_handle(&mut self) -> Result<(), Exception> {
        let new_handle = Writer::from_path(load_csv_path()?)?;
        self.file_handle = RwLock::new(new_handle);

        Ok(())
    }

    fn is_now(&self, alarm_time: (u32, u32, Weekday)) -> bool {
        let (h, m, wd) = alarm_time;

        let local = Local::now();
        let hour = local.hour();
        let minute = local.minute();
        let weekday = local.weekday();
        
        if hour == h && minute == m && weekday == wd {
            true
        } else {
            false
        }
    }
}