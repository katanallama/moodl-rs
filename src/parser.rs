use crate::models::course_details::ParseCourseDetails;
use anyhow::Result;
use html2md::parse_html;
use serde_json;
use std::fs::File as StdFile;
use std::io::Write;

fn convert_to_markdown(course_details: ParseCourseDetails) -> String {
    let mut markdown = String::new();

    // if let Some(course_id) = course_details.courseid {
    //     markdown.push_str(&format!("# Course ID: {}\n\n", course_id));
    // }

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
                markdown.push_str(&format!("\n{}\n", parse_html(module_description)));
            }

            // markdown.push_str("#### Contents\n");
            for content in &module.content {
                // if let Some(filename) = &content.content_filename {
                //     markdown.push_str(&format!("- Filename: {}\n", filename));
                // }
                // if let Some(fileurl) = &content.content_fileurl {
                //     markdown.push_str(&format!("- File URL: {}\n", fileurl));
                // }
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

    markdown
}

pub fn save_markdown_to_file(json_data: &str, file_path: &str) -> Result<()> {
    let parsed_course_details: ParseCourseDetails = serde_json::from_str(json_data)?;
    let markdown_data = convert_to_markdown(parsed_course_details);

    let mut file = StdFile::create(file_path)?;
    file.write_all(markdown_data.as_bytes())?;

    Ok(())
}

