use std::collections::HashSet;
use std::error::Error;
use std::fmt::{Debug, Display};

use chrono::prelude::*;
use config::{self, Config};

use crate::load_config;
use crate::Exception;

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

pub enum UnitTime {
    Hour,
    Minute,
}

impl Error for WeekdayException {}

/// Scheduler struct.
/// 
/// "update_scheduler()", "default()" and "is_now()" are 
/// the only public interfaces in the struct.
pub struct Scheduler {
    hour: u32,
    minute: u32,
    weekday: Weekday,
    keyword: HashSet<String>,
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
        };
        self.weekday = weekday?;

        Ok(())
    }

    fn update_keyword_and_email(&mut self, config: &Config) -> Result<(), Exception> {
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

        Ok(())
    }

    pub fn update_scheduler(&mut self) -> Result<(), Exception> {
        let config = load_config()?;
        self.update_time(&config)?;
        self.update_weekday(&config)?;
        self.update_keyword_and_email(&config)?;
        self.update_profile(&config)?;

        Ok(())
    }

    pub fn is_now(&mut self) -> bool {
        let local = Local::now();
        if local.weekday() == self.weekday && local.hour() == self.hour && local.minute() == self.minute {
            true
        } else {
            false
        }
    }
}