use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

/// Newtype for the hashmap.
pub struct Storage(Arc<Mutex<HashMap<String, Paper>>>);

impl Storage {
    pub fn new() -> Self {
        let map = HashMap::<String, Paper>::new();

        Self(Arc::new(Mutex::new(map)))
    }

    pub fn contains(&self, key: &str) -> bool {
        let storage_guard = self.lock().unwrap();
        storage_guard.contains_key(key)
    }

    pub fn push(&self, key: &str, value: Paper) {
        let mut storage_guard = self.lock().unwrap();
        storage_guard.insert(key.into(), value);
    }
}

/// Cloning a storage value returns a copy of the shared reference
/// to the same value.
impl Clone for Storage {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self))
    }
}

/// Dereferencing a storage value returns a reference to the shared 
/// reference to the same storage value.
impl Deref for Storage {
    type Target = Arc<Mutex<HashMap<String, Paper>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// All scraped papers are formatted to this struct and stored
/// in the storage.
#[derive(serde::Serialize)]
pub struct Paper {
    pub title: String,
    pub href: String,
    pub keyword: String,
    pub journal: String,
    pub date_published: String,
}

/// Pretty-print on the console for debugging.
impl Debug for Paper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, 
            "title: {}\nhref: {}\nkeyword: {}\njournal: {}\ndate_published: {}",
            self.title, self.href, self.keyword, self.journal, self.date_published,
        )
    }
}