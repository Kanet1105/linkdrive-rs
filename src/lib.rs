mod crawler;
mod scheduler;
mod storage;

use std::env::current_dir;
use std::path::PathBuf;

use config::{self, Config};
use lettre::{Message, SmtpTransport, Transport};
use lettre::message::{header::ContentType, Attachment};
use lettre::transport::smtp::authentication::Credentials;

use crawler::ChromeDriver;
use scheduler::Scheduler;

/// Type aliasing for Box<dyn std::error::Error> that is used globally.
pub type Exception = Box<dyn std::error::Error>;

/// The entry point of the app.
pub fn run_app() -> Result<(), Exception> {
    // Initialize the default scheduler as a mutable reference.
    let mut scheduler = Scheduler::default();

    // Initialize the Chrome web driver as a mutable reference.
    let mut web_driver = ChromeDriver::new()?;

    // Apply any changes made to "Settings.toml" in a loop.
    loop {
        // Update the scheduler and search.
        match scheduler.update_scheduler() {
            Err(e) => {
                dbg!(e); 
                continue
            },
            Ok(_) => {
                web_driver.search(&scheduler)?;
            },
        }
        std::thread::sleep(std::time::Duration::from_secs(60));
    }
}

/// Load configurations from the Settings.toml file located at
/// the program root directory.
fn load_config() -> Result<Config, Exception> {
    // The base path for configs ("./Settings.toml").
    let mut settings_path = current_dir()?;
    settings_path.push("Settings.toml");
    let settings_path_str = settings_path.to_str().unwrap();

    // Build the config file.
    let config = Config::builder()
        .add_source(config::File::with_name(settings_path_str))
        .build()?;

    Ok(config)
}

fn load_csv_path() -> Result<PathBuf, Exception> {
    let mut csv_path = current_dir()?;
    csv_path.push("Papers.csv");

    Ok(csv_path)
}

// fn send_email(id: &str, password: &str, email: &str) -> Result<(), Exception> {
//     // Set credentials for the SMTP server.
//     let creds = Credentials::new(id.to_string(), password.to_string());

//     // Set the csv file to send.
//     let file_name = "Papers.csv".to_string();
//     let file_body = fs::read(csv_path("Papers.csv")?)?;
//     let content_type = ContentType::parse("text/csv")?;
//     let attachment = Attachment::new(file_name).body(file_body, content_type);

//     // Build the message block.
//     let mut message = Message::builder()
//         .from(format!("Dongjae Park <{}@naver.com>", id).parse().unwrap())
//         .to(email.parse().unwrap())
//         .subject("SMTP Test")
//         .singlepart(attachment)
//         .unwrap();

//     // Open a remote connection to naver SMTP server.
//     let mailer = SmtpTransport::relay("smtp.naver.com")
//         .unwrap()
//         .credentials(creds)
//         .build();

//     // Send the email
//     match mailer.send(&message) {
//         Ok(_) => println!("Email sent successfully!"),
//         Err(e) => panic!("Could not send email: {:?}", e),
//     }

//     Ok(())
// }

// #[test]
// fn send_email_test() {
//     let id = value_to_string("profile", "id").unwrap();
//     let password = value_to_string("profile", "password").unwrap();

//     // let keyword = value_to_vec("default", "keyword").unwrap();
//     let email = value_to_string("default", "email").unwrap();

//     let file_name = "Papers.csv".to_string();
//     let file_body = fs::read("D:\\RustProjects\\linkdrive-rs\\Papers.csv").unwrap();
//     let content_type = ContentType::parse("text/csv").unwrap();
//     let attachment = Attachment::new(file_name).body(file_body, content_type);

//     let email = Message::builder()
//         .from(format!("Donghoon Lee <{}@naver.com>", &id).parse().unwrap())
//         .to(email.parse().unwrap())
//         .subject("Multiple Receivers Test")
//         .singlepart(attachment)
//         .unwrap();

//     let creds = Credentials::new(id, password);

//     // Open a remote connection to gmail
//     let mailer = SmtpTransport::relay("smtp.naver.com")
//         .unwrap()
//         .credentials(creds)
//         .build();

//     // Send the email
//     match mailer.send(&email) {
//         Ok(_) => println!("Email sent successfully!"),
//         Err(e) => panic!("Could not send email: {:?}", e),
//     }
// }