use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;

pub struct Storage(Arc<Haystack>);

impl Storage {
    pub fn new() -> Self {
        let haystack = Haystack::new();

        Self(Arc::new(haystack))
    }
}

impl Clone for Storage {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self))
    }
}

impl Deref for Storage {
    type Target = Arc<Haystack>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct Haystack {
    stack: HashMap<String, String>,
}

impl Haystack {
    pub fn new() -> Self {
        Self {
            stack: HashMap::<String, String>::new(),
        }
    }

    pub fn contains(&self, key: &str) -> bool {
        self.stack.contains_key(key)
    }
}

// #[derive(Debug)]
pub struct Paper {
    pub title: String,
    pub href: String,
    pub keyword: String,
    pub journal: String,
    pub date_published: String,
}

/// Pretty-print on console.
impl Debug for Paper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, 
            "title: {}\nhref: {}\nkeyword: {}\njournal: {}\ndate_published: {}",
            self.title, self.href, self.keyword, self.journal, self.date_published,
        )
    }
}