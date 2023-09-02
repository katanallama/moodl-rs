// models/course_contents.rs
//
use crate::models::response::CustomError;
use crate::process_result::ProcessResult;
use chrono::Utc;
use rusqlite::{Connection, Result};
use serde_derive::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize, Debug)]
pub struct CourseSection {
    pub id: i32,
    pub sectionid: Option<i64>,
    pub courseid: i32,
    pub name: String,
    pub modules: Vec<CourseModule>,
    pub summary: String,
    pub lastfetched: i64,
}

#[derive(Deserialize, Debug)]
pub struct CourseModule {
    pub id: i32,
    pub courseid: i32,
    pub moduleid: Option<i64>,
    pub modulename: String,
    pub content: String,
    pub lastfetched: i64,
}

#[derive(Deserialize, Debug)]
pub struct Assignment {
    pub id: i32,
    pub assignid: Option<i64>,
    pub cmid: Option<i64>, // course module id
    pub courseid: Option<i64>,
    pub content: String,
    pub lastfetched: i64,
}

#[derive(Deserialize, Debug)]
pub struct Grade {
    pub id: i32,
    pub gradeid: Option<i64>,
    pub courseid: Option<i64>,
    pub cmid: i64, // course module id
    pub content: String,
    pub lastfetched: i64,
}

#[derive(Deserialize, Debug)]
pub struct Page {
    pub id: i32,
    pub cmid: Option<i64>, // course module id
    pub courseid: Option<i64>,
    pub content: String,
    pub lastfetched: i64,
}

enum ContentType {
    Assignment,
    Content,
    Grade,
    Page,
}

pub fn process_pages(_conn: &Connection, content: &str) -> Result<ProcessResult, CustomError> {
    let (parsed, now) = prepare(content)?;
    let process_page = |page: &Value| -> Page {
        Page {
            id: 0,
            cmid: page["coursemodule"].as_i64(),
            courseid: page["course"].as_i64(),
            content: filter_data(page, ContentType::Page).to_string(),
            lastfetched: now,
        }
    };

    let page_list: Vec<Page> = process_array_items(&parsed, "pages", process_page);

    Ok(ProcessResult::Pages(page_list))
}

pub fn process_assignments(
    _conn: &Connection,
    content: &str,
) -> Result<ProcessResult, CustomError> {
    let (parsed, now) = prepare(content)?;
    let mut assignment_list: Vec<Assignment> = Vec::new();

    // Define a processor for assignments
    let process_assignment = |assignment: &Value| -> Assignment {
        Assignment {
            id: 0,
            assignid: assignment["id"].as_i64(),
            courseid: assignment["course"].as_i64(),
            cmid: assignment["cmid"].as_i64(),
            content: filter_data(assignment, ContentType::Assignment).to_string(),
            lastfetched: now,
        }
    };

    // For each course, process the assignments within
    let process_course_assignments = |course: &Value| -> Vec<Assignment> {
        process_array_items(course, "assignments", process_assignment)
    };

    for course in process_array_items(&parsed, "courses", process_course_assignments) {
        assignment_list.extend(course);
    }

    Ok(ProcessResult::Assigns(assignment_list))
}

pub fn process_grades(_conn: &Connection, content: &str) -> Result<ProcessResult, CustomError> {
    let (parsed, now) = prepare(content)?;
    let mut grade_list: Vec<Grade> = Vec::new();

    // Define a processor for grades
    let process_grade =
        |grade: &Value, courseid: i64| -> Result<Grade, CustomError> {
            Ok(Grade {
                id: 0,
                gradeid: Some(grade["id"].as_i64().ok_or_else(|| {
                    CustomError::TypeMismatch("Expected grade id to be i64".into())
                })?),
                courseid: Some(courseid),
                cmid: grade["cmid"].as_i64().unwrap_or_default(),
                content: filter_data(grade, ContentType::Grade).to_string(),
                lastfetched: now,
            })
        };

    // For each course, process the grades within
    let process_course_grades = |course: &Value| -> Result<Vec<Grade>, CustomError> {
        let courseid = course["courseid"]
            .as_i64()
            .ok_or_else(|| CustomError::TypeMismatch("Expected courseid to be i64".into()))?;

        let grades: Result<Vec<_>, _> =
            process_array_items(course, "gradeitems", |grade| process_grade(grade, courseid))
                .into_iter()
                .collect();

        grades
    };

    // Process all courses (potentially)
    let courses_grades: Result<Vec<Vec<Grade>>, CustomError> =
        process_array_items(&parsed, "usergrades", process_course_grades)
            .into_iter()
            .collect();

    for course_grades in courses_grades? {
        grade_list.extend(course_grades);
    }

    Ok(ProcessResult::Grades(grade_list))
}

fn process_module(now: i64, module: &Value) -> Result<CourseModule, CustomError> {
    if module["id"].is_null() || module["name"].is_null() {
        return Err(CustomError::MissingField(
            "Module missing id or name".into(),
        ));
    }

    Ok(CourseModule {
        id: 0,
        courseid: 0,
        moduleid: Some(
            module["id"]
                .as_i64()
                .ok_or_else(|| CustomError::TypeMismatch("Expected i64 for moduleid".into()))?,
        ),
        modulename: module["name"].to_string(),
        content: filter_data(module, ContentType::Content).to_string(),
        lastfetched: now,
    })
}

fn process_section(now: i64, section: &Value) -> Result<CourseSection, CustomError> {
    if section["id"].is_null() || section["name"].is_null() {
        return Err(CustomError::MissingField(
            "Section missing id or name".into(),
        ));
    }

    let c_modules: Vec<_> = if let Some(modules) = section["modules"].as_array() {
        modules
            .iter()
            .map(|module| process_module(now, module))
            .collect::<Result<Vec<_>, _>>()?
    } else {
        vec![]
    };

    Ok(CourseSection {
        id: 0,
        courseid: 0,
        sectionid: Some(
            section["id"]
                .as_i64()
                .ok_or_else(|| CustomError::TypeMismatch("Expected i64 for sectionid".into()))?,
        ),
        name: section["name"].to_string(),
        modules: c_modules,
        summary: section["summary"].to_string(),
        lastfetched: now,
    })
}

pub fn process_content(_conn: &Connection, content: &str) -> Result<ProcessResult, CustomError> {
    let parsed: Value = serde_json::from_str(content)?;
    let now = Utc::now().timestamp();

    let c_sections = if let Some(sections) = parsed.as_array() {
        sections
            .iter()
            .map(|section| process_section(now, section))
            .collect::<Result<Vec<_>, _>>()?
    } else {
        vec![]
    };

    Ok(ProcessResult::Content(c_sections))
}

fn prepare(content: &str) -> Result<(Value, i64), CustomError> {
    let parsed = serde_json::from_str(content)?;
    let now = Utc::now().timestamp();
    Ok((parsed, now))
}

fn process_array_items<T, F>(parsed_data: &Value, key: &str, processor: F) -> Vec<T>
where
    F: Fn(&Value) -> T,
{
    let mut items = Vec::new();

    if let Some(data_array) = parsed_data[key].as_array() {
        for item in data_array {
            items.push(processor(item));
        }
    }

    items
}


fn filter_data(content: &Value, content_type: ContentType) -> Value {
    match content_type {
        ContentType::Grade => {
            json!({
                "itemname": content["itemname"],
                "graderaw": content["graderaw"],
                "grademax": content["grademax"],
                "gradedate": content["gradedategraded"],
                "feedback": content["feedback"],
            })
        }
        ContentType::Content => {
            json!({
                "id": content["id"],
                "name": content["name"],
                "description": content["description"],
                "contents": [{
                    "type": content["contents"][0]["type"],
                    "filename": content["contents"][0]["filename"],
                    "fileurl": content["contents"][0]["fileurl"],
                    "timemodified": content["contents"][0]["timemodified"]
                }]
            })
        }
        ContentType::Assignment => {
            let introattachments: Vec<Value> = content["introattachments"]
                .as_array()
                .unwrap_or(&Vec::new())
                .iter()
                .map(|attachment| {
                    json!({
                        "filename": attachment["filename"],
                        "fileurl": attachment["fileurl"],
                        "timemodified": attachment["timemodified"],
                    })
                })
                .collect();

            json!({
                "name": content["name"],
                "duedate": content["duedate"],
                "cutoffdate": content["cutoffdate"],
                "timemodified": content["timemodified"],
                "intro": content["intro"],
                "introattachments": introattachments
            })
        }
        ContentType::Page => {
            let contentfiles: Vec<Value> = content["contentfiles"]
                .as_array()
                .unwrap_or(&Vec::new())
                .iter()
                .map(|file| {
                    json!({
                        "filename": file["filename"],
                        "fileurl": file["fileurl"],
                        "timemodified": file["timemodified"],
                    })
                })
                .collect();

            json!({
                "name": content["name"],
                "coursemodule": content["coursemodule"],
                "course": content["course"],
                "intro": content["intro"],
                "introfiles": content["introfiles"],
                "content": content["content"],
                "timemodified": content["timemodified"],
                "revision": content["revision"],
                "contentfiles": contentfiles
            })
        }
    }
}
