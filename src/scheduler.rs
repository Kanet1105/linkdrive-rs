use std::collections::HashSet;
use std::error::Error;
use std::fmt::{Debug, Display};
use std::fs;

use chrono::prelude::*;
use config::{self, Config};
use lettre::{Message, SmtpTransport, Transport};
use lettre::message::{header::ContentType, Attachment};
use lettre::transport::smtp::authentication::Credentials;

use crate::{load_config, load_csv_path};
use crate::Exception;

/// Scheduler struct.
/// 
/// Getter functions which take &self are the 
/// only public interfaces in the struct.
pub struct Scheduler {
    hour: u32,
    minute: u32,
    weekday: Weekday,
    keyword: HashSet<String>,
    email: String,
    id: Option<String>,
    password: Option<String>,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self {
            hour: 8,
            minute: 0,
            weekday: Weekday::Wed,
            keyword: HashSet::<String>::new(),
            email: String::new(),
            id: None,
            password: None,
        }
    }
}

impl Scheduler {
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
        self.parse_time(hh, UnitTime::Hour)?;
        self.parse_time(mm, UnitTime::Minute)?;

        Ok(())
    }

    fn parse_time(&mut self, time_str: &str, ut: UnitTime) -> Result<(), Exception> {
        match ut {
            UnitTime::Hour => {
                let hour = time_str.parse::<u32>()?;
                if hour >= 24 {
                    let message = "Set hour between 0 <= 'HH' < 24".to_string();
                    return Err(Box::new(TimeFormatException((message, hour.to_string()))))
                }
                self.hour = hour;
            },
            UnitTime::Minute => {
                let minute = time_str.parse::<u32>()?;
                if minute >= 60 {
                    let message = "Set minute between 0 <= 'MM' < 60".to_string();
                    return Err(Box::new(TimeFormatException((message, minute.to_string()))))
                }
                self.minute = minute;
            },
        }

        Ok(())
    }

    fn update_weekday(&mut self, config: &Config) -> Result<(), Exception> {
        let table = config.get_table("default")?;
        let weekday_value = table
            .get("weekday").unwrap()
            .to_string();

        let weekday = match weekday_value.as_str() {
            "Mon" => Ok(Weekday::Mon),
            "Tue" => Ok(Weekday::Tue),
            "Wed" => Ok(Weekday::Wed),
            "Thu" => Ok(Weekday::Thu),
            "Fri" => Ok(Weekday::Fri),
            "Sat" => Ok(Weekday::Sat),
            "Sun" => Ok(Weekday::Sun),
            _ => Err(Box::new(WeekdayException(weekday_value))),
        }?;
        self.weekday = weekday;

        Ok(())
    }

    fn update_keyword_and_email(&mut self, config: &Config) -> Result<(), Exception> {
        let table = config.get_table("default")?;

        // Set keywords.
        let keyword: HashSet<String> = table
            .get("keyword").unwrap()
            .clone()
            .into_array()?
            .iter()
            .map(|x| { x.to_string() })
            .collect();
        self.keyword = keyword;

        // Set an email
        let email: String = table
            .get("email").unwrap()
            .to_string();
        self.email = email;

        Ok(())
    }

    /// This is where you can fail to send an email if you do not 
    /// set your SMTP settings properly.
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

        if &id == "" || &password == "" {
            let message = "Email ID / Password field is empty.".to_string();
            return Err(Box::new(ProfileException(message)))
        }

        self.id = Some(id);
        self.password = Some(password);

        Ok(())
    }

    /// Apply changes in Settings.toml file to the scheduler
    /// during the runtime.
    pub fn update_scheduler(&mut self) -> Result<(u32, u32, Weekday), Exception> {
        let config = load_config()?;
        self.update_time(&config)?;
        self.update_weekday(&config)?;
        self.update_keyword_and_email(&config)?;
        self.update_profile(&config)?;

        Ok(self.alarm_time())
    }

    /// Keyword getter.
    pub fn keyword<'a>(&'a self) -> &'a HashSet<String> {
        &self.keyword
    }

    /// Alarm time getter.
    pub fn alarm_time(&self) -> (u32, u32, Weekday) {
        (self.hour, self.minute, self.weekday.clone())
    }

    /// Send an email.
    pub fn send_email(&mut self, local_time: &str) -> Result<(), Exception> {
        // Set credentials for SMTP protocol.
        let id = self.id.clone().unwrap();
        let password = self.password.clone().unwrap();
        let credentials = Credentials::new(id.to_string(), password);

        // Set the csv file.
        let file_name = "Papers.csv".to_string();
        let file_body = fs::read(load_csv_path()?)?;
        let content_type = ContentType::parse("text/csv")?;
        let attachment = Attachment::new(file_name).body(file_body, content_type);
        
        // Build the message block.
        let email = self.email.clone();
        let message = Message::builder()
            .from(format!("Crawler <{}@naver.com>", id).parse().unwrap())
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