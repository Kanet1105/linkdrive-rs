use std::error::Error;
use std::fmt::{Debug, Display};

use chrono::prelude::*;
use config::{self, Config};

use super::load_config;
use super::Exception;

pub struct TimeFormatException((String, String));

impl Debug for TimeFormatException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buffer = format!(
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
        let mut buffer = format!(
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
/// "set_scheduler()", "default()" and "is_now()" are the
/// only public functions that interfaces with a user.
pub struct Scheduler {
    hour: u32,
    minute: u32,
    weekday: Weekday,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self {
            hour: 8,
            minute: 0,
            weekday: Weekday::Wed,
        }
    }
}

impl Scheduler {
    fn set_time(&mut self, config: &Config) -> Result<(), Exception> {
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

    fn set_weekday(&mut self, config: &Config) -> Result<(), Exception> {
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

    pub fn set_scheduler(&mut self) -> Result<(), Exception> {
        let config = load_config()?;
        self.set_time(&config)?;
        self.set_weekday(&config)?;

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