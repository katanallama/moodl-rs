use crate::models::{course::CourseSection, grades::GradeItem};
use chrono::NaiveDateTime;
use eyre::Result;
use fancy_regex::Regex;
use html2md::parse_html;
use std::{fs::File as StdFile, io::Write};

pub fn parse_grades(grades: Vec<GradeItem>) -> String {
    let mut max_name_len = 9;

    grades.iter().for_each(|grade| {
        if let Some(name) = &grade.itemname {
            if !name.trim().is_empty() {
                max_name_len = max_name_len.max(name.len());
            }
        }
    });

    let max_grade_len = 5;
    let max_date_len = 19;
    let max_range_len = 12;

    let mut markdown = format!(
        "| {:<name_width$} | {:<grade_width$} | {:<date_width$} | {:<range_width$} |\n",
        "Item Name",
        "Grade",
        "Graded On",
        "Grade Range",
        name_width = max_name_len,
        grade_width = max_grade_len,
        date_width = max_date_len,
        range_width = max_range_len
    );
    markdown.push_str(&format!(
        "|{:-<name_dashes$}|{:-<grade_dashes$}|{:-<date_dashes$}|{:-<range_dashes$}|\n",
        "",
        "",
        "",
        "",
        name_dashes = max_name_len + 2,
        grade_dashes = max_grade_len + 2,
        date_dashes = max_date_len + 2,
        range_dashes = max_range_len + 2
    ));

    grades.into_iter().for_each(|grade| {
        if let Some(name) = grade.itemname.as_ref().map(|s| remove_emojis(s)) {
            if name.trim().is_empty() {
                return;
            }

            let grade_val = if let Some(grade) = &grade.graderaw {
                format!("{:#?}", grade)
            } else {
                String::from("N/A")
            };

            let grade_date = if let Some(date) = &grade.gradedategraded {
                format!("{}", parse_date(*date))
            } else {
                String::from("N/A")
            };

            let grade_range = format!("{:#?} - {:#?}", grade.grademin, grade.grademax);

            markdown.push_str(&format!(
                "| {:<name_width$} | {:<grade_width$} | {:<date_width$} | {:<range_width$} |\n",
                name,
                grade_val,
                grade_date,
                grade_range,
                name_width = max_name_len,
                grade_width = max_grade_len,
                date_width = max_date_len,
                range_width = max_range_len
            ));
        }
    });

    markdown
}

pub fn parse_course(course: Vec<CourseSection>) -> String {
    let mut markdown = String::new();

    course.into_iter().for_each(|section| {
        log::debug!("Section name: {}", section.name);

        markdown.push_str(&format!("# {}", section.name));
        let summary = clean_html(&section.summary);
        markdown.push_str(&format!("\n{}", parse_html(&summary)));
        // markdown.push_str(&format!("{}\n\n", summary));

        let mut is_first_module = true;

        section.modules.into_iter().for_each(|module| {
            log::debug!("Module name: {}", remove_emojis(&module.name));

            if is_first_module {
                markdown.push_str(&format!("## {}", remove_emojis(&module.name)));
                is_first_module = false;
            } else {
                markdown.push_str(&format!("### {}", remove_emojis(&module.name)));
            }

            if let Some(desc) = &module.description {
                let desc = clean_html(&desc);
                if desc.trim() != module.name.trim() {
                    markdown.push_str(&format!("\n{}\n", parse_html(&desc)));
                    // markdown.push_str(&format!("\n{}\n", desc));
                }
            }

            match module.contents {
                Some(files) => {
                    files.into_iter().for_each(|mut file| {
                        if let Some(ref mut name) = file.filename {
                            *name = remove_emojis(&name);
                            if let Some(path) = &file.filepath {
                                log::debug!("Module file: {} at {}", name, path);
                                markdown.push_str(&format!("\n[{}]", name));
                                markdown.push_str(&format!("({})\n", path));
                            } else if let Some(url) = &file.fileurl {
                                markdown.push_str(&format!("\n[{}]", name));
                                markdown.push_str(&format!("({})\n", url));
                            }
                        }
                    });
                }
                _ => (),
            }
            markdown.push_str(&format!("\n\n"));
        });
    });
    markdown
}

pub fn save_markdown_to_file(parsed_course: String, file_path: &str) -> Result<()> {
    let file_path = format!("{}.md", file_path);
    let mut file = StdFile::create(file_path)?;
    Ok(file.write_all(parsed_course.as_bytes())?)
}

pub fn clean_html(html: &String) -> String {
    let mut clean_html = html.clone();

    let tags_to_remove = vec![
        r#" dir="ltr\""#,
        r#" lang="EN-US\""#,
        r#" class="\""#,
        r#" style="text-align: left;\""#,
        r"<br>",
        r"</ br>",
        r"<br />",
        r"\r",
    ];

    for tag in tags_to_remove {
        let re = Regex::new(tag).unwrap();
        clean_html = re.replace_all(&clean_html, "").to_string();
    }

    let pattern = r"<(\w+)[^>]*>\s*</\1>";
    let re = Regex::new(pattern).unwrap();

    let mut previous_html;
    loop {
        previous_html = clean_html.clone();
        clean_html = re.replace_all(&clean_html, "").to_string();
        if clean_html == previous_html {
            break;
        }
    }

    clean_html = remove_line_breaks(&clean_html);

    clean_html
}

fn parse_date(timestamp: i64) -> String {
    NaiveDateTime::from_timestamp_opt(timestamp, 0)
        .expect("Invalid timestamp")
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}

fn remove_emojis(s: &str) -> String {
    s.chars().filter(|&c| c.is_ascii()).collect()
}

fn remove_line_breaks(input: &str) -> String {
    let re = Regex::new(r"\s+").unwrap();
    re.replace_all(input, " ").to_string()
}
