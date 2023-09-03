// ui/parser.rs
use crate::models::course_content::CourseModule;
use crate::models::response::CustomError;
use html2md::parse_html;
use regex;
use regex::Regex;
use rusqlite::{params, Connection};
use serde_json::Value;

pub struct ParsedModule {
    pub name: String,
    pub description: String,
    pub url: Option<String>,
}

pub struct ParsedPage {
    pub name: String,
    pub description: String,
    pub url: Option<String>,
}

fn fetch_modules(
    conn: &Connection,
    course_id: i32,
) -> Result<Vec<CourseModule>, CustomError> {
    let mut stmt = conn.prepare(
        "SELECT id, courseid, moduleid, modulename, content, lastfetched FROM Modules WHERE courseid = ?1",
    )?;
    let rows = stmt.query_map(params![course_id], |row| {
        Ok(CourseModule {
            id: row.get(0)?,
            courseid: row.get(1)?,
            moduleid: row.get(2)?,
            modulename: row.get(3)?,
            content: row.get(4)?,
            lastfetched: row.get(5)?,
        })
    })?;

    let mut modules = Vec::new();
    for row_result in rows {
        match row_result {
            Ok(module) => modules.push(module),
            Err(e) => return Err(CustomError::from(e)), // Assuming you have a from implementation for your error type
        }
    }

    Ok(modules)
}


pub fn fetch_and_parse_modules(
    conn: &Connection,
    course_id: i32,
) -> Result<Vec<ParsedModule>, CustomError> {
    let rows = fetch_modules(conn, course_id)?;
    parse_modules(rows)
}


fn parse_modules(rows: Vec<CourseModule>) -> Result<Vec<ParsedModule>, CustomError> {
    let mut parsed_modules = Vec::new();

    for module in rows {
        match parse_module(&module) {
            Ok(parsed) => parsed_modules.push(parsed),
            Err(e) => eprintln!("Error parsing module: ",),
        }
    }

    Ok(parsed_modules)
}

fn parse_module(module: &CourseModule) -> Result<ParsedModule, CustomError> {
    let json_data: Value = serde_json::from_str(&module.content)?;

    let description = parse_description(&json_data)?;
    let url = extract_url(&json_data);

    Ok(ParsedModule {
        name: module.modulename.replace(r#"""#, r#""#),
        description,
        url,
    })
}

fn parse_description(json_data: &Value) -> Result<String, CustomError> {
    if let Ok(unescaped_html) = serde_json::from_str::<String>(&json_data["description"].to_string()) {
        Ok(parse_html(&clean_description(&unescaped_html)))
    } else {
        Ok(String::new())
    }
}

fn clean_description(description: &str) -> String {
    let patterns = vec![
        (r#"<br>"#, ""),
        (r#"#### (.*?) ####"#, "# $1"),
        (r#"### (.*?) ###"#, "# $1"),
        (r#"\*\*\*\*\* (.*?) \*\*\*\*\*"#, "**$1**"),
        (r#"(\*\*\*\*\*)|(\*\*\*\*)"#, "**"),
    ];

    let mut cleaned = description.to_string();

    for (pattern, replace_with) in patterns {
        let re = Regex::new(pattern).unwrap();
        cleaned = re.replace_all(&cleaned, replace_with).to_string();
    }

    cleaned
}

fn extract_url(json_data: &Value) -> Option<String> {
    json_data["contents"]
        .as_array()
        .and_then(|contents| contents.get(0))
        .and_then(|content| content["fileurl"].as_str())
        .map(|s| s.to_string())
}





fn print_modules(modules: &[ParsedModule]) {
    for module in modules {
        println!("\nModule Name: {}", module.name);
        println!("Description: {}", module.description);
        if let Some(url) = &module.url {
            println!("URL: {}", url);
        }
    }
}

pub fn fetch_and_print_modules(conn: &Connection, course_id: i32) -> Result<(), CustomError> {
    let modules = fetch_and_parse_modules(&conn, course_id)?;
    print_modules(&modules);
    // let _ = ui(&modules);

    Ok(())
}
