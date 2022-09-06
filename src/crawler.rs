use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::ffi::OsString;
use std::mem;
use std::sync::Arc;
use std::time::Duration;

use super::Exception;
use super::storage::{Paper, Storage};

use csv::{Writer, WriterBuilder};
use headless_chrome::{Browser, Element, LaunchOptionsBuilder, Tab};
use rayon::prelude::*;

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
    storage: RefCell<Storage>,
    keyword_set: RefCell<HashSet<String>>,
    file_path: String,
}

impl ChromeDriver {
    /// The function initializes the web driver client with a read-only javascript "Tab" object.
    /// 
    /// [WARNING]
    /// 
    /// Although "Arc<Tab>" seems to be thread-safe, the Tab object is actually a web api call
    /// that returns a shared reference to the current window handle. Javascript Window object
    /// can be mutated at any point without the Rust implementation of interior mutability.
    pub fn new(file_path: &str) -> Result<Self, Exception> {
        let user_agent = OsString::from("--user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.0.0 Safari/537.36");
        let options = LaunchOptionsBuilder::default()
            .args(vec![&user_agent])
            .headless(false)
            .build()?;
        let browser = Browser::new(options)?;
        let main_tab = browser.wait_for_initial_tab()?;

        Ok(Self {
            browser,
            main_tab,
            domain_string: "https://www.sciencedirect.com/".into(),
            base_query_string: "https://www.sciencedirect.com/search?qs=".into(),
            blank_token: "%20".into(),
            max_indices_per_page: 100,
            storage: RefCell::new(Storage::new()),
            keyword_set: RefCell::new(HashSet::<String>::new()),
            file_path: file_path.into(),
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
    pub fn search(&self, new_keyword: HashSet<String>) -> Result<(), Exception> {
        // Create a new storage to replace the current one with.
        let new_storage = Storage::new();

        let outer_selector = "#srp-results-list";
        let last_element = format!("#srp-results-list > ol > li:nth-child({})", self.max_indices_per_page);

        // Scrape the page with initialized query strings.
        for keyword in &new_keyword {
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
            let item_list = result_list.wait_for_elements("li")?;

            // Parallel parse() execution.
            self.parse(item_list, keyword, &self.domain_string, &new_storage)?;
        }
        self.update(new_storage, new_keyword)?;

        Ok(())
    }

    /// Multi-threaded parser utilizing ["rayon"].
    fn parse(
        &self, item_list: Vec<Element>, keyword: &str, domain: &str, new_storage: &Storage
    ) -> Result<(), Exception> {
        // Parse items in the list.
        item_list
            .par_iter()
            .for_each(|item| {
                // Get a clone of the new storage for the multi-threaded context.
                let storage = new_storage.clone();

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
                        (href, tokens[3].to_string())
                    };

                    // Build the paper struct.
                    let paper = Paper {
                        title: elements[0].get_inner_text().unwrap(),
                        href,
                        keyword: keyword.into(),
                        journal: elements[1].get_inner_text().unwrap(),
                    };

                    // Insert the paper in the new storage with the uid as its key.
                    storage.push(&uid, paper);
                }
            });

        Ok(())
    }

    /// Update is none other than overwriting the previous storage.
    /// It does not care whether two storages are the same or not.
    fn update(&self, new_storage: Storage, new_keyword: HashSet<String>) -> Result<(), Exception> {
        let mut storage_mutref = self.storage.borrow_mut();
        let prev_storage = mem::replace(&mut *storage_mutref, new_storage);

        let mut keyword_mutref = self.keyword_set.borrow_mut();
        let prev_keyword = mem::replace(&mut *keyword_mutref, new_keyword);

        // todo! => modularize update().
        
        // let current_storage = self.storage.borrow().lock().unwrap();
        // let current_keyword = self.keyword_set.borrow();

        // for (uid, paper) in &*current_storage {
        //     if !prev_storage.contains(uid) && prev_keyword.contains(&paper.keyword) {

        //     }

        //     // Test pretty-printing the paper.
        //     println!("UID : {}", &uid);
        //     println!("{:?}", &paper);
        //     println!("======================================================");
        // }

        Ok(())
    }
}