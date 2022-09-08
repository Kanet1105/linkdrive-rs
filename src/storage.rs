use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::{Debug, Display};
use std::fs::{self, File};
use std::mem;
use std::sync::RwLock;

use chrono::prelude::*;
use config::Config;
use csv::Writer;
use lettre::{Message, SmtpTransport, Transport};
use lettre::message::{header::ContentType, Attachment};
use lettre::transport::smtp::authentication::Credentials;

use crate::{load_config, load_csv_path};
use crate::Exception;

pub struct Storage {
    keyword: RwLock<HashSet<String>>,
    storage: RwLock<HashMap<String, Paper>>,
    up_storage: RwLock<HashMap<String, Paper>>,
    settings: RwLock<Settings>,
    file_handle: RwLock<Writer<File>>,
}

impl Storage {
    pub fn new() -> Self {
        let keyword = HashSet::<String>::new();
        let storage = HashMap::<String, Paper>::new();
        let up_storage = HashMap::<String, Paper>::new();
        let settings = Settings::new().unwrap();
        let file_handle = Writer::from_path(load_csv_path().unwrap()).unwrap();

        Self {
            keyword: RwLock::new(keyword),
            storage: RwLock::new(storage),
            up_storage: RwLock::new(up_storage),
            settings: RwLock::new(settings),
            file_handle: RwLock::new(file_handle),
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
        writer.insert(href.to_string(), value.clone());

        // Only write to the file when the keyword has already been added,
        // but the paper by the key is not in the hashmap.
        let reader = self.keyword.read().unwrap();
        if !self.contains_key(&href) && !reader.contains(&keyword) {
            true
        } else {
            false
        }
    }

    /// Utilizes [std::mem::take] and [std::mem::replace] to replace the 
    /// current value with the new value.
    pub fn update(&self, new_keyword: HashSet<String>) {
        let _ = mem::replace(
            &mut *self.keyword.write().unwrap(), 
            new_keyword
        );

        let new_storage = mem::take(&mut *self.up_storage.write().unwrap());
        let _ = mem::replace(
            &mut *self.storage.write().unwrap(), 
            new_storage,
        );
    }

    /// Utilizes [std::mem::replace] to replace the current file handle
    /// with the new one after sending an email.
    pub fn new_file_handle(&self) -> Result<(), Exception> {
        let new_file = Writer::from_path(load_csv_path()?)?;
        let _ = mem::replace(
            &mut *self.file_handle.write().unwrap(), 
            new_file
        );
        Ok(())
    }

    /// Update the changes applied to the "Settings.toml" file.
    pub fn update_settings(&self) -> Result<(), Exception> {
        let mut writer = self.settings.write().unwrap();
        writer.update_settings()?;
        Ok(())
    }

    pub fn keyword_from_settings(&self) -> HashSet<String> {
        let reader = self.settings.read().unwrap();
        reader.keyword.clone()
    }

    pub fn time_from_settings(&self) -> (u32, u32, Weekday) {
        let reader = self.settings.read().unwrap();
        (reader.hour, reader.minute, reader.weekday)
    }

    pub fn write_to_file(&self, paper: Paper) -> Result<(), Exception> {
        let mut writer = self.file_handle.write().unwrap();
        writer.serialize(paper)?;
        writer.flush()?;
        Ok(())
    }

    pub fn send_email(&self, local_time: &str) -> Result<(), Exception> {
        let writer = self.settings.write().unwrap();
        writer.send_email(local_time)?;
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

/// Setter for key-value pairs in "Settings.toml" files.
/// id and password are no longer optional fields. They
/// need to be filled out in order to use the program.
pub struct Settings {
    pub keyword: HashSet<String>,
    pub email: String,
    pub hour: u32,
    pub minute: u32,
    pub weekday: Weekday,
    id: String,
    password: String,
}

impl Settings {
    pub fn new() -> Result<Self, Exception> {
        let mut me = Self {
            keyword: HashSet::<String>::new(),
            email: String::new(),
            hour: 8,
            minute: 30,
            weekday: Weekday::Sun,
            id: "".into(),
            password: "".into(),
        };
        me.update_settings()?;
        Ok(me)
    }

    /// Apply changes in Settings.toml file to the scheduler
    /// during the runtime.
    pub fn update_settings(&mut self) -> Result<(), Exception> {
        let config = load_config()?;
        self.update_keyword(&config)?;
        self.update_email(&config)?;
        self.update_time(&config)?;
        self.update_weekday(&config)?;
        self.update_profile(&config)?;
        Ok(())
    }

    /// It is a list of strings.
    /// ```
    /// keyword = ["X", "Y", "Z"]
    /// ```
    /// The below format is also allowed in TOML.
    /// ```
    /// keyword = [
    ///     "X",
    ///     "Y",
    ///     "Z",
    /// ]
    /// ```
    fn update_keyword(&mut self, config: &Config) -> Result<(), Exception> {
        let table = config.get_table("default")?;
        let keyword: HashSet<String> = table
            .get("keyword").unwrap()
            .clone()
            .into_array()?
            .iter()
            .map(|x| { x.to_string() })
            .collect();
        self.keyword = keyword;
        Ok(())
    }

    /// The regular email address string.
    /// ```
    /// email = "zombiedelah@gmail.com"
    /// ```
    fn update_email(&mut self, config: &Config) -> Result<(), Exception> {
        let table = config.get_table("default")?;
        let email: String = table
            .get("email").unwrap()
            .to_string();
        self.email = email;
        Ok(())
    }

    /// The hour and the minute to receive the email on.
    /// 
    /// 0 <= "HH" < 24
    /// 
    /// 0 <= "MM" < 60
    /// ```
    /// time = "HH:MM" 
    /// ```
    fn update_time(&mut self, config: &Config) -> Result<(), Exception> {
        let table = config.get_table("default")?;
        let alarm_time = table
            .get("time").unwrap()
            .to_string();

        // Missing splicer ':'.
        if !alarm_time.contains(':') {
            let message = "Missing splicer ':' in the time format.".to_string();
            return Err(Box::new(TimeFormatException((message, alarm_time.into()))));
        }

        // Wrong format or range.
        let (hh, mm) = alarm_time.split_once(':').unwrap();
        self.hour = self.parse_time(hh, UnitTime::Hour)?;
        self.minute = self.parse_time(mm, UnitTime::Minute)?;
        Ok(())
    }

    fn parse_time(&mut self, time_str: &str, ut: UnitTime) -> Result<u32, Exception> {
        match ut {
            UnitTime::Hour => {
                let hour = time_str.parse::<u32>()?;
                if hour >= 24 {
                    let message = "Set hour between 0 <= 'HH' < 24".to_string();
                    return Err(Box::new(TimeFormatException((message, hour.to_string()))))
                }

                Ok(hour)
            },
            UnitTime::Minute => {
                let minute = time_str.parse::<u32>()?;
                if minute >= 60 {
                    let message = "Set minute between 0 <= 'MM' < 60".to_string();
                    return Err(Box::new(TimeFormatException((message, minute.to_string()))))
                }

                Ok(minute)
            },
        }
    }

    /// Choose one of the weekday to receive an email on.
    /// ```
    /// weekday = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
    /// ```
    fn update_weekday(&mut self, config: &Config) -> Result<(), Exception> {
        let table = config.get_table("default")?;
        let weekday_value = table
            .get("weekday").unwrap()
            .to_string();

        self.weekday = match weekday_value.as_str() {
            "Mon" => Ok(Weekday::Mon),
            "Tue" => Ok(Weekday::Tue),
            "Wed" => Ok(Weekday::Wed),
            "Thu" => Ok(Weekday::Thu),
            "Fri" => Ok(Weekday::Fri),
            "Sat" => Ok(Weekday::Sat),
            "Sun" => Ok(Weekday::Sun),
            _ => Err(Box::new(WeekdayException(weekday_value))),
        }?;
        Ok(())
    }

    /// /// # Warning
    /// Never upload the "Settings.toml" file with user id and password!
    /// 
    /// ```
    /// id = "user id"
    /// password = "user password"
    /// ```
    fn update_profile(&mut self, config: &Config) -> Result<(), Exception> {
        let table = config.get_table("profile")?;
        let (id, password): (String, String) = {
            let id: String = table
                .get("id").unwrap()
                .to_string();
            let password: String = table
                .get("password").unwrap()
                .to_string();
            (id, password)
        };
        
        // Never allow an empty field.
        if &id == "" || &password == "" {
            let message = "Email ID / Password field is empty.".to_string();
            return Err(Box::new(ProfileException(message)))
        }
        self.id = id;
        self.password = password;
        Ok(())
    }

    /// Send an email.
    fn send_email(&self, local_time: &str) -> Result<(), Exception> {
        // Set credentials for SMTP protocol.
        let credentials = Credentials::new(
            self.id.to_string(), 
            self.password.to_string()
        );

        // Set the csv file.
        let file_name = "Papers.csv".to_string();
        let file_body = fs::read(load_csv_path()?)?;
        let content_type = ContentType::parse("text/csv")?;
        let attachment = Attachment::new(file_name).body(file_body, content_type);
        
        // Build the message block.
        let email = self.email.clone();
        let message = Message::builder()
            .from(format!("Crawler <{}@naver.com>", &self.id).parse().unwrap())
            .to(email.parse().unwrap())
            .subject("SMTP Test")
            .singlepart(attachment)?;

        // Open a remote connection to naver SMTP server.
        let mailer = SmtpTransport::relay("smtp.naver.com")?
            .credentials(credentials)
            .build();

        match mailer.send(&message) {
            Ok(_) => {
                println!("Message sent at [{}]", local_time);
            },
            Err(e) => { dbg!(e); },
        }
        Ok(())
    }
}

pub struct TimeFormatException((String, String));

impl Debug for TimeFormatException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let buffer = format!(
            "\n\t{}\n\
            \ttime = {} is not a valid time format.\n\
            \ttime = 'HH:MM' is the valid format.",
            &self.0.0, &self.0.1
        );
        write!(f, "{}", buffer)
    }
}

impl Display for TimeFormatException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let buffer = format!(
            "\n\t{}\n\
            \ttime = {} is not a valid time format.\n\
            \ttime = 'HH:MM' is the valid format.",
            &self.0.0, &self.0.1
        );
        write!(f, "{}", buffer)
    }
}

impl Error for TimeFormatException {}

pub struct WeekdayException(String);

impl Debug for WeekdayException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, 
            "\n\tweekday = '{}' is not a valid weekday format.\nChoose from\n\
            \t'Mon'\n\
            \t'Tue'\n\
            \t'Wed'\n\
            \t'Thu'\n\
            \t'Fri'\n\
            \t'Sat'\n\
            \t'Sun'\n",
            &self.0
        )
    }
}

impl Display for WeekdayException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, 
            "\n\tweekday = '{}' is not a valid weekday format.\nChoose from\n\
            \t'Mon'\n\
            \t'Tue'\n\
            \t'Wed'\n\
            \t'Thu'\n\
            \t'Fri'\n\
            \t'Sat'\n\
            \t'Sun'\n",
            &self.0
        )
    }
}

impl Error for WeekdayException {}

pub struct ProfileException(String);

impl Debug for ProfileException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\n\t{}", &self.0)
    }
}

impl Display for ProfileException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\n\t{}", &self.0)
    }
}

impl Error for ProfileException {}

pub enum UnitTime {
    Hour,
    Minute,
}