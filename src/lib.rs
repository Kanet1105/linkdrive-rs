mod crawler;
mod storage;

use std::cell::RefCell;
use std::env::current_dir;
use std::path::PathBuf;
use std::rc::Rc;
use std::thread::sleep;
use std::time::Duration;

use crawler::ChromeDriver;

/// Type aliasing for Box<dyn std::error::Error> that is used globally.
pub type Exception = Box<dyn std::error::Error>;

/// The entry point of the app.
pub fn run_app() -> Result<(), Exception> {
    tracing_subscriber::fmt()
        .pretty()
        .init();

    // Initialize the crawler and the flag as a mutable reference.
    let web_driver = ChromeDriver::new()?;
    tracing::info!("Initialize the Chrome web driver");
    
    let crawler = Rc::new(RefCell::new(web_driver));
    let flag = Rc::new(RefCell::new(false));
    tracing::info!("running..");

    loop {
        let mut crawler_mut = crawler.borrow_mut();
        crawler_mut.avoid_timeout()?;
        match crawler_mut.is_now() {
            Ok(bool_value) => {
                if bool_value {
                    // Set the event off only when
                    // "bool_value" == true && "flag" == false.
                    let mut flag_mut = flag.borrow_mut();
                    if !(*flag_mut) {
                        match crawler_mut.search() {
                            Ok(()) => {}
                            Err(e) => {
                                dbg!(e);
                            }
                        }
                        *flag_mut = true;
                        continue;
                    } else {
                        sleep(Duration::from_millis(1600));
                        continue;
                    }
                } else {
                    // Otherwise, set the flag back to false.
                    let mut flag_mut = flag.borrow_mut();
                    *flag_mut = false;
                    sleep(Duration::from_millis(1600));
                }
            }
            Err(e) => {
                dbg!(e);
                sleep(Duration::from_millis(1600));
            }
        }
    }
}

fn load_csv_path() -> Result<PathBuf, Exception> {
    let mut csv_path = current_dir()?;
    csv_path.push("Papers.csv");
    Ok(csv_path)
}
