use crate::models::course_details::ParseCourseDetails;
// use select::document::Document;
// use select::node::Node;
// use select::predicate::{Name, Predicate};
// use select::predicate::Predicate;
// use select::*;
use eyre::Result;
use html2md::{parse_html, parse_html_extended};
use scraper::{ElementRef, Html, Selector};
use serde_json;
use std::{
    fs::{self, File as StdFile},
    io::Write,
};

fn convert_to_markdown(course_details: ParseCourseDetails) -> String {
    let mut markdown = String::new();

    for section in course_details.sections {
        if let Some(section_name) = &section.section_name {
            markdown.push_str(&format!("# {}\n", section_name));
        }
        if let Some(section_summary) = &section.section_summary {
            markdown.push_str(&format!("{}\n", parse_html(section_summary)));
        }
        // if let Some(section_lastfetched) = &section.section_lastfetched {
        //     markdown.push_str(&format!("Last Fetched: {}\n", section_lastfetched));
        // }

        for module in section.modules {
            if let Some(module_name) = &module.module_name {
                markdown.push_str(&format!("\n## {}\n", module_name));
            }
            if let Some(module_description) = &module.module_description {
                let clean_desc = clean_html(&module_description);
                let parsed_md = parse_html_extended(&clean_desc);
                markdown.push_str(&format!("\n{}\n", parsed_md));
            }

            // markdown.push_str("#### Contents\n");
            for content in &module.content {
                if let Some(content_filename) = &content.content_filename {
                    if let Some(content_fileurl) = &content.content_fileurl {
                        markdown.push_str(&format!("\n[{}]", content_filename));
                        markdown.push_str(&format!("({})\n", content_fileurl));
                    }
                }
            }

            // markdown.push_str("#### Pages\n");
            for page in &module.pages {
                if let Some(page_name) = &page.page_name {
                    markdown.push_str(&format!("\n### {}\n", page_name));
                }
                // if let Some(page_intro) = &page.page_intro {
                //     markdown.push_str(&format!("- Page Intro: {}\n", parse_html(page_intro)));
                // }
                if let Some(page_content) = &page.page_content {
                    markdown.push_str(&format!("{}\n", parse_html(page_content)));
                }

                // markdown.push_str("##### Files\n");
                for file in &page.files {
                    if let Some(file_filename) = &file.file_filename {
                        if let Some(file_fileurl) = &file.file_fileurl {
                            markdown.push_str(&format!("\n[{}]", file_filename));
                            markdown.push_str(&format!("({})\n", file_fileurl));
                        }
                    }
                }
            }

            markdown.push_str("\n");
        }
    }

    if let Some(course_id) = course_details.courseid {
        log::info!("Parsed course {:?}", course_id);
    }

    markdown
}

fn clean_html(html: &str) -> String {
    let parsed_html = Html::parse_fragment(html);
    let selector_h1 = Selector::parse("h1").unwrap();
    let selector_h2 = Selector::parse("h2").unwrap();
    let selector_h3 = Selector::parse("h3").unwrap();
    let selector_h4 = Selector::parse("h4").unwrap();
    let selector_h5 = Selector::parse("h5").unwrap();
    let selector_h6 = Selector::parse("h6").unwrap();

    let mut cleaned_html = String::new();
    for node in parsed_html.tree.root().children() {
        if let Some(element) = ElementRef::wrap(node) {
            if !(element.select(&selector_h1).next().is_some()
                || element.select(&selector_h2).next().is_some()
                || element.select(&selector_h3).next().is_some()
                || element.select(&selector_h4).next().is_some()
                || element.select(&selector_h5).next().is_some()
                || element.select(&selector_h6).next().is_some())
            {
                cleaned_html.push_str(&element.html());
            }
        }
    }

    cleaned_html
}

pub fn save_markdown_to_file(json_data: &str, file_path: &str) -> Result<()> {
    let parsed_course_details: ParseCourseDetails = serde_json::from_str(json_data)?;
    let markdown_data = convert_to_markdown(parsed_course_details);

    let file_path_with_extension = format!("{}.md", file_path);

    if let Some(parent_dir) = std::path::Path::new(&file_path_with_extension).parent() {
        if !parent_dir.exists() {
            fs::create_dir_all(parent_dir)?;
        }
    }

    let mut file = StdFile::create(file_path_with_extension)?;
    file.write_all(markdown_data.as_bytes())?;

    Ok(())
}
