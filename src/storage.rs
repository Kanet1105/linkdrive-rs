use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::fs::File;
use std::mem;
use std::sync::RwLock;

use csv::Writer;

use crate::Exception;
use crate::load_csv_path;

pub struct Storage {
    keyword: HashSet<String>,
    storage: RwLock<HashMap<String, Paper>>,
    up_storage: RwLock<HashMap<String, Paper>>,
    buffer: RwLock<Writer<File>>,
}

impl Storage {
    pub fn new() -> Self {
        let keyword = HashSet::<String>::new();
        let storage = HashMap::<String, Paper>::new();
        let up_storage = HashMap::<String, Paper>::new();
        let buffer = Writer::from_path(load_csv_path().unwrap()).unwrap();

        Self {
            keyword,
            storage: RwLock::new(storage),
            up_storage: RwLock::new(up_storage),
            buffer: RwLock::new(buffer),
        }
    }
    
    pub fn contains_key(&self, key: &str) -> bool {
        let reader = self.storage.read().unwrap();
        reader.contains_key(key)
    }

    /// Write to the new storage which will later update the current one.
    /// It takes a tuple argument consisting of ("keyword", "href") and 
    /// returns true if the new paper is uploaded.
    pub fn insert(&self, key: (String, String), value: Paper) -> bool {
        let (keyword, href) = key;
        let mut writer = self.up_storage.write().unwrap();
        writer.insert(href.to_string(), value);

        // Only write to the file when the keyword has already been added,
        // but the paper by the key is not in the hashmap.
        if !self.contains_key(&href) && !self.keyword.contains(&keyword) {
            true // Write to the file when it returns true.
        } else {
            false
        }
    }

    /// Utilizes [std::mem::take] and [std::mem::replace] to replace the 
    /// current value with the new value.
    pub fn update(&mut self, new_keyword: &HashSet<String>) {
        self.keyword = new_keyword.clone();
        let updated_storage = mem::take(&mut self.up_storage);
        let _ = mem::replace(&mut self.storage, updated_storage);
    }

    /// Write the paper to a file.
    pub fn write(&self, paper: Paper) -> Result<(), Exception> {
        let mut writer = self.buffer.write().unwrap();
        writer.serialize(paper)?;

        Ok(())
    }
}

#[derive(Clone, serde::Serialize)]
pub struct Paper {
    pub keyword: String,
    pub title: String,
    pub journal: String,
    pub href: String,
}

/// Pretty-print on the console for debugging.
impl Debug for Paper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, 
            "\n\ttitle: {}\n\thref: {}\n\tkeyword: {}\n\tjournal: {}\n\
            ==================================================",
            self.title, self.href, self.keyword, self.journal,
        )
    }
}