use ical::property::Property;
use std::{error::Error, fmt};
use std::io::BufReader;
use chrono::{NaiveDate, NaiveTime, IsoWeek, Datelike};
use lazy_regex::{regex_captures};

#[derive(Debug, Clone, Copy)]
pub enum EventCategory {
    Default,
    Yellow,
    Green,
    Red,
    Blue
}

#[derive(Debug, Clone)]
pub struct Event {
    pub category: EventCategory,
    pub date: NaiveDate,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub description: String,
    pub summary: String,
    pub location: String,

    pub module_name: Option<String>
}

#[derive(Debug, Clone)]
pub struct Timetable {
    pub events: Vec<Event>
}

#[derive(Debug)]
pub enum GetTimetableError {
    NotFound,
    EmptyTimetable
}

impl Timetable {
    pub fn by_week(&self, week: IsoWeek) -> Vec<Event> {
        return self.events.iter()
            .filter(|e| e.date.iso_week() == week)
            .map(|e| e.clone())
            .collect();
    }
    pub fn max_end_time(&self) -> Option<NaiveTime> {
        return self.events.iter()
            .map(|e| e.end_time)
            .max();
    }
}

impl Error for GetTimetableError {}

impl fmt::Display for GetTimetableError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use GetTimetableError::*;
        match self {
            NotFound => write!(f, "Timetable not found"),
            EmptyTimetable => write!(f, "Timetable is empty")
        }
    }
}

fn guess_module_name(summary: &str) -> Option<String> {
    let captures = regex_captures!(r"^\w\d{3}\w\d{3} (.+)", summary);
    if let Some((_, module_name)) = captures {
        return Some(module_name.into());
    }
    None
}

pub trait TimetableGetter {
    fn get(&self, vidko: &str) -> Result<Timetable, GetTimetableError>;
}

#[derive(Default)]
pub struct BlockingTimetableGetter {}
impl TimetableGetter for BlockingTimetableGetter {
    fn get(&self, vidko: &str) -> Result<Timetable, GetTimetableError> {
        fn find_property<'a>(properties: &'a [Property], name: &str) -> Result<&'a Property, GetTimetableError> {
            for prop in properties {
                if prop.name == name {
                    return Ok(prop);
                }
            }
            panic!("Property '{}' not found", name);
        }

        let resp = ureq::get(&format!("https://uais.cr.ktu.lt/ktuis/tv_rprt2.ical1?p={}&t=basic.ics", vidko))
            .call()
            .map_err(|_| GetTimetableError::NotFound)?;

        let mut reader = ical::IcalParser::new(BufReader::new(resp.into_reader()));
        let cal = reader.next();
        if cal.is_none() {
            return Err(GetTimetableError::NotFound)
        }
        let cal = cal.unwrap().unwrap();
        if cal.events.is_empty() {
            return Err(GetTimetableError::EmptyTimetable);
        }

        let mut timetable = Timetable { events: vec![] };
        for event in cal.events {
            let category_prop = find_property(&event.properties, "CATEGORIES")?;
            let start_prop = find_property(&event.properties, "DTSTART")?;
            let end_prop = find_property(&event.properties, "DTEND")?;
            let description_prop = find_property(&event.properties, "DESCRIPTION")?;
            let summary_prop = find_property(&event.properties, "SUMMARY")?;
            let location_prop = find_property(&event.properties, "LOCATION")?;

            let mut category = EventCategory::Default;
            if let Some(category_str) = &category_prop.value {
                category = match category_str.as_str() {
                    "Yellow Category" => EventCategory::Yellow,
                    "Green Category" => EventCategory::Green,
                    "Red Category" => EventCategory::Red,
                    "Blue Category" => EventCategory::Blue,
                    _ => EventCategory::Default
                };
            }
            let start_str = start_prop.value.clone().unwrap();
            let end_str = end_prop.value.clone().unwrap();
            let (start_date, start_time) = start_str.split_once('T').unwrap();
            let (_end_date, end_time) = end_str.split_once('T').unwrap();
            let summary = summary_prop.value.clone().unwrap();

            timetable.events.push(Event {
                category,
                date: NaiveDate::parse_from_str(start_date, "%Y%m%d").unwrap(),
                start_time: NaiveTime::parse_from_str(start_time, "%H%M%S").unwrap(),
                end_time: NaiveTime::parse_from_str(end_time, "%H%M%S").unwrap(),
                description: description_prop.value.clone().unwrap(),
                module_name: guess_module_name(&summary),
                summary,
                location: location_prop.value.clone().unwrap()
            })
        }

        timetable.events.sort_by_key(|event| (event.date, event.start_time));

        Ok(timetable)
    }
}

pub struct DummyTimetableGetter {
    timetable: Timetable
}
impl DummyTimetableGetter {
    #[allow(unused)]
    pub fn new(timetable: Timetable) -> Self {
        Self { timetable }
    }
}
impl TimetableGetter for DummyTimetableGetter {
    fn get(&self, _vidko: &str) -> Result<Timetable, GetTimetableError> {
        Ok(self.timetable.clone())
    }
}