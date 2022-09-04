mod configuration;
mod crawler;
mod storage;

use configuration::validate_paths;
use crawler::ChromeDriver;

/// Type aliasing for Box<dyn std::error::Error> that is used globally.
pub type Exception = Box<dyn std::error::Error>;

/// The entry point of the app.
pub fn run_app() -> Result<(), Exception> {
    let web_driver = ChromeDriver::new()?;

    loop {
        // validate if all config files necessary to run the program is
        // in the right paths.
        validate_paths()?;
        web_driver.search()?;
    }
}

#[test]
fn csv_writer() {
    use csv::WriterBuilder;
    let mut csv_writer = WriterBuilder::new()
        .from_path("new.csv")
        .unwrap();

    let p1 = Paper {
        title: "p1".into(),
        href: "https://google.com".into(),
        keyword: "test1".into(),
        journal: "test_journal1".into(),
        date_published: "today".into(),
    };

    let p2 = Paper {
        title: "p2".into(),
        href: "https://naver.com".into(),
        keyword: "test2".into(),
        journal: "test_journal2".into(),
        date_published: "tomorrow".into(),
    };

    csv_writer.serialize(p1).unwrap();
    csv_writer.serialize(p2).unwrap();
}

#[derive(serde::Serialize)]
/// All scraped papers are formatted to this struct and stored
/// in the storage.
pub struct Paper {
    pub title: String,
    pub href: String,
    pub keyword: String,
    pub journal: String,
    pub date_published: String,
}

/// Pretty-print on the console for debugging.
impl std::fmt::Debug for Paper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, 
            "title: {}\nhref: {}\nkeyword: {}\njournal: {}\ndate_published: {}",
            self.title, self.href, self.keyword, self.journal, self.date_published,
        )
    }
}