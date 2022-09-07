mod crawler;
mod scheduler;
mod storage;

use std::collections::HashSet;
use std::fs;
use std::env::current_dir;

use crawler::ChromeDriver;
use scheduler::Scheduler;

use chrono::prelude::*;
use config::{self, Config};
use lettre::{Message, SmtpTransport, Transport};
use lettre::message::{header::ContentType, Attachment};
use lettre::transport::smtp::authentication::Credentials;

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
        // Check scheduler.
        scheduler.set_scheduler()?;

        // Get "profile" table and extract "id" and "password" fields.
        let id = value_to_string("profile", "id")?;
        let password = value_to_string("profile", "password")?;
        if id == "" || password == "" {
            panic!("Empty 'id' | 'password' for email.\nCheck the profile table in Settings.toml.");
        }

        // Get "default" table and extract "keyword" and "email" fields.
        let keyword = value_to_hashset("default", "keyword")?;
        let email = value_to_string("default", "email")?;

        // Search the web.
        web_driver.search(keyword)?;

        // Send an email to the user.
        if scheduler.is_now() {
            // send_email(&id, &password, &email)?;
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

/// Convert from "Value" to "String" for the given table and the key.
fn value_to_string(table: &str, key: &str) -> Result<String, Exception> {
    let config = load_config()?;
    let config_table = config.get_table(table)?;
    let extracted: String = config_table
        .get(key)
        .unwrap()
        .to_string();

    Ok(extracted)
}

/// Convert from "Value" to "HashSet<String>" for the given table and the key.
fn value_to_hashset(table: &str, key: &str) -> Result<HashSet<String>, Exception> {
    let config = load_config()?;
    let config_table = config.get_table(table)?;
    let extracted: HashSet<String> = config_table
        .get(key).unwrap()
        .clone()
        .into_array()?
        .iter()
        .map(|x| { x.to_string() })
        .collect();
    
    Ok(extracted)
}

fn csv_path(file_name: &str) -> Result<String, Exception> {
    // Set the csv file path to write the list of new papers to.
    let mut csv_file_path = current_dir()?;
    csv_file_path.push(file_name);
    let file_path = csv_file_path
        .to_str()
        .unwrap()
        .to_string();

    Ok(file_path)
}

fn send_email(id: &str, password: &str, email: &str) -> Result<(), Exception> {
    // Set credentials for the SMTP server.
    let creds = Credentials::new(id.to_string(), password.to_string());

    // Set the csv file to send.
    let file_name = "Papers.csv".to_string();
    let file_body = fs::read(csv_path("Papers.csv")?)?;
    let content_type = ContentType::parse("text/csv")?;
    let attachment = Attachment::new(file_name).body(file_body, content_type);

    // Build the message block.
    let mut message = Message::builder()
        .from(format!("Dongjae Park <{}@naver.com>", id).parse().unwrap())
        .to(email.parse().unwrap())
        .subject("SMTP Test")
        .singlepart(attachment)
        .unwrap();

    // Open a remote connection to naver SMTP server.
    let mailer = SmtpTransport::relay("smtp.naver.com")
        .unwrap()
        .credentials(creds)
        .build();

    // Send the email
    match mailer.send(&message) {
        Ok(_) => println!("Email sent successfully!"),
        Err(e) => panic!("Could not send email: {:?}", e),
    }

    Ok(())
}

#[test]
fn send_email_test() {
    let id = value_to_string("profile", "id").unwrap();
    let password = value_to_string("profile", "password").unwrap();

    // let keyword = value_to_vec("default", "keyword").unwrap();
    let email = value_to_string("default", "email").unwrap();

    let file_name = "Papers.csv".to_string();
    let file_body = fs::read("D:\\RustProjects\\linkdrive-rs\\Papers.csv").unwrap();
    let content_type = ContentType::parse("text/csv").unwrap();
    let attachment = Attachment::new(file_name).body(file_body, content_type);

    let email = Message::builder()
        .from(format!("Donghoon Lee <{}@naver.com>", &id).parse().unwrap())
        .to(email.parse().unwrap())
        .subject("Multiple Receivers Test")
        .singlepart(attachment)
        .unwrap();

    let creds = Credentials::new(id, password);

    // Open a remote connection to gmail
    let mailer = SmtpTransport::relay("smtp.naver.com")
        .unwrap()
        .credentials(creds)
        .build();

    // Send the email
    match mailer.send(&email) {
        Ok(_) => println!("Email sent successfully!"),
        Err(e) => panic!("Could not send email: {:?}", e),
    }
}