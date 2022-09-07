use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::mem;
use std::sync::RwLock;

pub struct Storage {
    keyword: HashSet<String>,
    storage: RwLock<HashMap<String, Paper>>,
    up_storage: RwLock<HashMap<String, Paper>>,
}

impl Storage {
    pub fn new() -> Self {
        let keyword = HashSet::<String>::new();
        let storage = HashMap::<String, Paper>::new();
        let up_storage = HashMap::<String, Paper>::new();

        Self {
            keyword,
            storage: RwLock::new(storage),
            up_storage: RwLock::new(up_storage),
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

        if !self.contains_key(&href) && self.keyword.contains(&keyword) {
            true
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
            "\ttitle: {}\n\thref: {}\n\tkeyword: {}\n\tjournal: {}\n\
            ==================================================\n",
            self.title, self.href, self.keyword, self.journal,
        )
    }
}