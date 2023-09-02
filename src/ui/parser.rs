// ui/parser.rs
use crate::models::course_content::CourseModule;
use crate::models::response::CustomError;
// use crate::ui::tui::*;
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

pub fn fetch_and_parse_modules(
    conn: &Connection,
    course_id: i32,
) -> Result<Vec<ParsedModule>, CustomError> {
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

    let mut parsed_modules = Vec::new();

    for module in rows {
        match module {
            Ok(m) => {
                let json_data: Value = serde_json::from_str(&m.content)?;

                let description = if let Ok(unescaped_html) =
                    serde_json::from_str::<String>(&json_data["description"].to_string())
                {
                    parse_html(&unescaped_html.replace(r#"<br>"#, r#""#))
                } else {
                    String::new()
                };

                let url = json_data["contents"]
                    .as_array()
                    .and_then(|contents| contents.get(0))
                    .and_then(|content| content["fileurl"].as_str())
                    .map(|s| s.to_string());

                // Replace headers surrounded by #### with #
                let header_re = Regex::new(r#"#### (.*?) ####"#).unwrap();
                let cleaned_headers = header_re.replace_all(&description, "# $1");

                let header_re = Regex::new(r#"### (.*?) ###"#).unwrap();
                let cleaned_headers = header_re.replace_all(&cleaned_headers, "# $1");

                // Replace excessive asterisks around bold text with just two asterisks
                let bold_re = Regex::new(r#"\*\*\*\*\* (.*?) \*\*\*\*\*"#).unwrap();
                let cleaned_bold = bold_re.replace_all(&cleaned_headers, "**$1**");

                // Handle other specific cases
                let other_re = Regex::new(r#"(\*\*\*\*\*)|(\*\*\*\*)"#).unwrap();
                let description = other_re.replace_all(&cleaned_bold, "**");
                parsed_modules.push(ParsedModule {
                    name: m.modulename.replace(r#"""#, r#""#),
                    description: description.to_string(),
                    url,
                });
            }
            Err(e) => {
                eprintln!("Error fetching module: {}", e);
            }
        }
    }

    Ok(parsed_modules)
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
