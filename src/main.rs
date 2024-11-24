use chrono::NaiveDate;
use colored::*;
use ics::properties::{Description, DtEnd, DtStart, Location, Summary};
use ics::{Event, ICalendar};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::process::exit;

// Data structures for deserialisation
#[derive(Serialize, Deserialize, Debug)]
struct Course {
    course: String,
    allocation: Allocation,
}

#[derive(Serialize, Deserialize, Debug)]
struct Allocation {
    class: Class,
}

#[derive(Serialize, Deserialize, Debug)]
struct Class {
    consult: Vec<Consultation>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Consultation {
    instructors: Vec<String>,
    weeks: String,
    day: String,
    start: String,
    end: String,
    mode: String,
    location: Option<String>,
}

// Function to parse the weeks string into a vector of week numbers
fn parse_weeks(weeks_str: &str) -> Vec<u32> {
    let mut weeks = Vec::new();

    for part in weeks_str.split(',') {
        let part = part.trim();
        if part.contains('-') {
            let mut range_parts = part.split('-');
            let start = range_parts.next().unwrap().trim().parse::<u32>();
            let end = range_parts.next().unwrap().trim().parse::<u32>();

            match (start, end) {
                (Ok(start), Ok(end)) => {
                    for week in start..=end {
                        weeks.push(week);
                    }
                }
                _ => {
                    eprintln!("{}", format!("Invalid week range: {}", part).red());
                    continue;
                }
            }
        } else {
            match part.parse::<u32>() {
                Ok(week) => weeks.push(week),
                Err(_) => {
                    eprintln!("{}", format!("Invalid week number: {}", part).red());
                    continue;
                }
            }
        }
    }

    weeks
}

// Function to read and parse tutors.json into a HashMap
fn read_tutors(filename: &str) -> HashMap<String, String> {
    let tutor_list = fs::read_to_string(filename).expect("Failed to read tutors.json");
    serde_json::from_str(&tutor_list).expect("Failed to parse tutors.json")
}

// Function to get and verify the tutor's zID
fn get_verified_zid(tutors: &HashMap<String, String>) -> String {
    loop {
        print!("{}", "Enter your zID: ".blue());
        io::stdout().flush().expect("Failed to flush stdout");

        let mut zid = String::new();
        io::stdin()
            .read_line(&mut zid)
            .expect("Failed to read line");

        let zid = zid.trim().to_string();

        match tutors.get(&zid) {
            Some(name) => {
                print!(
                    "{}",
                    format!("Verify this is your name: {} [Y/N] ", name).yellow()
                );
                io::stdout().flush().expect("Failed to flush stdout");

                let mut response = String::new();
                io::stdin()
                    .read_line(&mut response)
                    .expect("Failed to read line");

                match response.trim().to_lowercase().as_str() {
                    "y" | "yes" => return zid,
                    "n" | "no" => {
                        println!("{}", "Please re-enter your zID.".red());
                        continue;
                    }
                    _ => {
                        println!("{}", "Invalid response. Please enter Y or N.".red());
                        continue;
                    }
                }
            }
            None => {
                println!("{}", format!("No tutor found with zID: {}", zid).red());
                println!("{}", "Please check your zID and try again.".red());
                continue;
            }
        }
    }
}

// Function to read and parse allocations.json into a vector of Course
fn read_allocations(filename: &str) -> Vec<Course> {
    let file_content = fs::read_to_string(filename).expect("Failed to read allocations.json");
    serde_json::from_str(&file_content).expect("Failed to parse allocations.json")
}

// Function to generate the calendar events
fn generate_calendar(courses: &[Course], zid: &str) -> (ICalendar<'static>, bool) {
    let mut calendar = ICalendar::new("2.0", "rust-ics");
    let mut found_allocations = false;

    // Determined Start Date Manually
    let start_date =
        NaiveDate::parse_from_str("2024-09-09", "%Y-%m-%d").expect("Invalid start date format");

    for course in courses {
        for consultation in &course.allocation.class.consult {
            if consultation.instructors.contains(&zid.to_string()) {
                found_allocations = true;

                let weeks = parse_weeks(&consultation.weeks);

                for week in weeks {
                    let week_start_date = start_date + chrono::Duration::weeks((week - 1) as i64);

                    let day_offset = match consultation.day.as_str() {
                        "Mon" => 0,
                        "Tue" => 1,
                        "Wed" => 2,
                        "Thu" => 3,
                        "Fri" => 4,
                        "Sat" => 5,
                        "Sun" => 6,
                        _ => {
                            eprintln!("{}", format!("Invalid day: {}", consultation.day).red());
                            continue;
                        }
                    };

                    let consultation_date = week_start_date + chrono::Duration::days(day_offset);

                    let dt_start = format!(
                        "{}T{}00",
                        consultation_date.format("%Y%m%d"),
                        consultation.start.replace(":", "")
                    );
                    let dt_end = format!(
                        "{}T{}00",
                        consultation_date.format("%Y%m%d"),
                        consultation.end.replace(":", "")
                    );

                    let uid = format!(
                        "{}-{}-{}-{}-week{}",
                        course.course, consultation.day, consultation.start, zid, week
                    );

                    let mut event = Event::new(uid, dt_start.clone());
                    event.push(DtStart::new(dt_start));
                    event.push(DtEnd::new(dt_end));
                    event.push(Summary::new(format!("{} Help Session", course.course)));
                    event.push(Description::new(format!(
                        "Mode: {}, Weeks: {}",
                        consultation.mode, consultation.weeks
                    )));
                    let location = consultation
                        .location
                        .clone()
                        .unwrap_or_else(|| "Online".to_string());
                    event.push(Location::new(location));

                    calendar.add_event(event);
                }
            }
        }
    }

    (calendar, found_allocations)
}

fn main() {
    let tutors = read_tutors("src/data/tutors.json");

    let zid = get_verified_zid(&tutors);

    let courses = read_allocations("src/data/allocations.json");

    let (calendar, found_allocations) = generate_calendar(&courses, &zid);

    if !found_allocations {
        println!("{}", format!("No allocations found for zID: {}", zid).red());
        exit(0);
    }

    let downloads_path = dirs::download_dir()
        .expect("Failed to locate the Downloads directory")
        .join("my_allocations.ics");

    match fs::write(&downloads_path, calendar.to_string()) {
        Ok(_) => println!(
            "{}",
            format!(
                "Calendar file 'my_allocations.ics' generated successfully at: {}",
                downloads_path.display()
            )
            .green()
        ),
        Err(e) => eprintln!("{}", format!("Failed to write .ics file: {}", e).red()),
    }
}
