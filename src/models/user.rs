// models/user.rs
//
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::prelude::*;

use crate::models::response::CustomError;
use crate::process_result::ProcessResult;
use chrono::Utc;
use rusqlite::{Connection, Result};
use serde_derive::Deserialize;
use serde_json::Value;

#[derive(Deserialize, Debug)]
pub struct User {
    pub id: i64,
    pub content: String,
    pub privkey: String,
    pub url: String,
    pub wstoken: String,
    pub lastfetched: i64,
}

#[derive(Deserialize, Debug)]
pub struct Course {
    pub id: i64,
    pub courseid: Option<i64>,
    pub shortname: String,
    pub fullname: String,
    pub lastfetched: i64,
}

pub fn write_course_conf(courses: Vec<Course>) -> Result<ProcessResult, CustomError> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open("moodl-rs-id")?;

    let mut course_map = HashMap::new();
    for course in &courses {
        if let Some(course_id) = course.courseid {
            course_map.insert(course_id, &course.shortname);
        }
    }

    file.write_all(b"Course Map: {\n")?;
    for (course_id, course_name) in &course_map {
        file.write_all(format!("    {}: {},\n", course_id, course_name).as_bytes())?;
    }
    file.write_all(b"}\n")?;

    Ok(ProcessResult::Courses(courses))
}

pub fn process_courses(_conn: &Connection, content: &str) -> Result<ProcessResult, CustomError> {
    let parsed: Value = serde_json::from_str(content)?;

    let mut course_list: Vec<Course> = Vec::new();
    let now = Utc::now().timestamp();

    if let Some(courses) = parsed.as_array() {
        for course in courses {
            let course = Course {
                id: 0,
                courseid: course["id"].as_i64(),
                shortname: course["shortname"].to_string(),
                fullname: course["fullname"].to_string(),
                lastfetched: now,
            };
            course_list.push(course);
        }
    }

    Ok(ProcessResult::Courses(course_list))
}

pub fn process_user(_conn: &Connection, content: &str) -> Result<ProcessResult, CustomError> {
    let parsed: Value = serde_json::from_str(content)?;
    let now = Utc::now().timestamp();

    if let Some(user_id) = parsed["userid"].as_i64() {
        let user = User {
            id: user_id,
            content: parsed.to_string(),
            privkey: parsed["userprivateaccesskey"].to_string(),
            url: "".to_string(),
            wstoken: "".to_string(),
            lastfetched: now,
        };

        return Ok(ProcessResult::User(user));
    }

    Ok(ProcessResult::None)
}
