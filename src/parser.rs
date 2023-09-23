use crate::models::{course::CourseSection, grades::GradeItem};
use chrono::NaiveDateTime;
use eyre::Result;
use fancy_regex::{Captures, Regex};
use scraper::Html;
use std::{fs::File as StdFile, io::Write};

trait Parser {
    fn parse_text(&mut self, text: &str);
    fn parse_start_tag(&mut self, tag: &str, attrs: &[(String, String)]);
    fn parse_end_tag(&mut self, tag: &str);
}

struct MarkdownParser {
    output: String,
}

impl MarkdownParser {
    fn new() -> Self {
        Self {
            output: String::new(),
        }
    }
}

impl Parser for MarkdownParser {
    fn parse_text(&mut self, text: &str) {
        self.output.push_str(text);
    }

    fn parse_start_tag(&mut self, tag: &str, attrs: &[(String, String)]) {
        match tag {
            "h1" => self.output.push_str("\n# "),
            "h2" => self.output.push_str("\n## "),
            "h3" => self.output.push_str("\n### "),
            "h4" => self.output.push_str("\n#### "),
            "h5" => self.output.push_str("\n##### "),
            "p" => (),
            "ul" => (),
            "li" => self.output.push_str("* "),
            "b" | "strong" => self.output.push_str("**"),
            "i" | "em" => self.output.push_str("_"),
            "a" => {
                if let Some(href) = attrs.iter().find(|&&(ref name, _)| name == "href") {
                    self.output.push_str(&format!("[{}](", href.1));
                }
            }
            _ => (),
        }
    }

    fn parse_end_tag(&mut self, tag: &str) {
        match tag {
            "h1" | "h2" | "h3" | "h4" | "h5" => self.output.push_str("\n"),
            "p" | "li" => self.output.push('\n'),
            "b" | "strong" => self.output.push_str("**"),
            "i" | "em" => self.output.push_str("_"),
            "a" => self.output.push_str(")"),
            _ => (),
        }
    }
}

fn traverse(element: scraper::ElementRef, parser: &mut dyn Parser) {
    let tag_name = element.value().name.local.as_ref();
    let attrs: Vec<(String, String)> = element
        .value()
        .attrs
        .iter()
        .map(|(name, value)| (name.local.to_string(), value.to_string()))
        .collect();

    parser.parse_start_tag(tag_name, &attrs);

    for node in element.children() {
        if node.value().is_text() {
            if let Some(text_content) = node.value().as_text() {
                parser.parse_text(text_content);
            }
        } else if node.value().is_element() {
            let child_element = scraper::ElementRef::wrap(node).unwrap();
            traverse(child_element, parser);
        }
    }

    parser.parse_end_tag(tag_name);
}

fn parse_html(html: &str) -> String {
    let fragment = Html::parse_fragment(html);

    let mut parser = MarkdownParser::new();

    for child in fragment.tree.root().children() {
        if let Some(element) = scraper::ElementRef::wrap(child) {
            traverse(element, &mut parser);
        }
    }

    // println!("{}", parser.output);
    parser.output
}

fn is_variation_of(short: &str, long: &str) -> bool {
    if !short.is_empty() || !long.is_empty() {
        let short_cleaned = short.trim_end_matches("...");
        let long_cleaned = long.replace("**", "");
        return long_cleaned.contains(short_cleaned);
    } else {
        return false;
    }
}

pub fn parse_course(course: Vec<CourseSection>) -> String {
    let mut markdown = String::new();

    course.into_iter().for_each(|section| {
        log::debug!("Section name: {}", section.name);

        markdown.push_str(&format!("# {}\n", section.name));
        let summary = clean_html(&section.summary);
        markdown.push_str(&format!("{}\n", parse_html(&summary)));

        section.modules.into_iter().for_each(|module| {
            log::debug!("Module name: {:#?}", remove_emojis(&module.name));

            let mut cleaned_desc: String = "".to_string();
            if let Some(desc) = &module.description {
                cleaned_desc = desc.to_string();
            }

            cleaned_desc = clean_html(&cleaned_desc);
            let parsed_desc = parse_html(&cleaned_desc);
            let module_name = remove_emojis(&module.name);

            // Only add name if it is not a variation of the name
            if is_variation_of(&module_name, &parsed_desc) {
                // markdown.push_str(&format!("\n## {}\n", module_name));
                markdown.push_str(&format!("{}", parsed_desc));
            } else {
                markdown.push_str(&format!("\n## {}\n", module_name));
                markdown.push_str(&format!("{}", parsed_desc));
            }
            // }

            if let Some(files) = &module.contents {
                files.into_iter().for_each(|file| {
                    if let Some(name) = &file.filename {
                        let clean_name = remove_emojis(&name);
                        if let Some(path) = &file.filepath {
                            log::debug!("Module file: {} at {}", clean_name, path);
                            markdown.push_str(&format!("\n[{}]", clean_name));
                            markdown.push_str(&format!("({})\n", path));
                        } else {
                            if let Some(url) = &file.fileurl {
                                markdown.push_str(&format!("\n[{}]", clean_name));
                                markdown.push_str(&format!("({})\n", url));
                            }
                        }
                    }
                });
            }
        });
        markdown.push_str("\n\n");
    });
    markdown
}

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
        "# Grades: \n\n| {:<name_width$} | {:<date_width$} | {:<range_width$} | {:<grade_width$} |\n",
        "Item Name",
        "Graded On",
        "Range",
        "Grade",
        name_width = max_name_len,
        date_width = max_date_len,
        range_width = max_range_len,
        grade_width = max_grade_len,
    );
    markdown.push_str(&format!(
        "|{:-<name_dashes$}|{:-<date_dashes$}|{:-<range_dashes$}|{:-<grade_dashes$}|\n",
        "",
        "",
        "",
        "",
        name_dashes = max_name_len + 2,
        date_dashes = max_date_len + 2,
        range_dashes = max_range_len + 2,
        grade_dashes = max_grade_len + 2,
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
                "| {:<name_width$} | {:<date_width$} | {:<range_width$} | {:<grade_width$} |\n",
                name,
                grade_date,
                grade_range,
                grade_val,
                name_width = max_name_len,
                date_width = max_date_len,
                range_width = max_range_len,
                grade_width = max_grade_len,
            ));
        }
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
        r#" "background-color: rgb(255, 207, 53);\""#,
        r#" style="text-align: left;\""#,
        r#" style="text-align: center;\""#,
        r"<br>",
        r"no-overflow",
        r"<span>",
        r"</span>",
        r"</ br>",
        r"<br />",
        r#" class="\""#,
        r"<div>",
        r"</div>",
        r"&nbsp;",
        // r"\r",
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
    clean_html = decrease_header_level(&clean_html);

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

fn decrease_header_level(input: &str) -> String {
    let re = Regex::new(r"<h(\d)>").unwrap();
    re.replace_all(input, |caps: &Captures| {
        let level: u32 = caps.get(1).unwrap().as_str().parse().unwrap();
        if level > 1 {
            format!("<h{}>", level - 1)
        } else {
            format!("<h{}>", level)
        }
    })
    .to_string()
}
