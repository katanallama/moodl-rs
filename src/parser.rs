use crate::models::course_details::ParseCourseDetails;
use eyre::Result;
use serde_json;
use std::{
    fs::{self, File as StdFile},
    io::Write,
};

fn convert_to_markdown(course_details: ParseCourseDetails) -> String {
    let mut markdown = String::new();

    for section in course_details.sections {
        if let Some(section_name) = &section.section_name {
            markdown.push_str(&format!("# {}", section_name));
        }
        if let Some(section_summary) = &section.section_summary {
            markdown.push_str(&format!("\n{}", section_summary));
        }

        for module in section.modules {
            if let Some(module_name) = &module.module_name {
                markdown.push_str(&format!("\n## {}\n", module_name));
            }
            if let Some(module_description) = &module.module_description {
                if !markdown.contains(module_description) {
                    markdown.push_str(&format!("{}\n\n", module_description));
                }
            }
            for page in module.pages {
                if let Some(page_content) = &page.page_content {
                    if !markdown.contains(page_content) {
                        markdown.push_str(&format!("{}\n\n", page_content));
                    }
                }
                for file in &page.files {
                    if let Some(file_filename) = &file.file_filename {
                        if let Some(file_localpath) = &file.file_localpath {
                            if let Some(stripped_localpath) = file_localpath.strip_prefix("out/") {
                                markdown.push_str(&format!("\n[{}]", file_filename));
                                markdown.push_str(&format!("({})\n\n", stripped_localpath));
                            }
                        } else if let Some(file_fileurl) = &file.file_fileurl {
                            markdown.push_str(&format!("\n[{}]", file_filename));
                            markdown.push_str(&format!("({})\n\n", file_fileurl));
                        }
                    }
                }
            }
            for content in &module.content {
                if let Some(content_filename) = &content.content_filename {
                    if !content_filename.contains("index.html") {
                        if let Some(content_localpath) = &content.content_localpath {
                            if let Some(stripped_localpath) = content_localpath.strip_prefix("out/")
                            {
                                markdown.push_str(&format!("[{}]", content_filename));
                                markdown.push_str(&format!("({})\n\n", stripped_localpath));
                            }
                        } else if let Some(content_fileurl) = &content.content_fileurl {
                            markdown.push_str(&format!("[{}]", content_filename));
                            markdown.push_str(&format!("({})\n\n", content_fileurl));
                        }
                    }
                }
            }
        }
        markdown.push_str("\n---");
        markdown.push_str("\n\n");
    }

    if let Some(course_id) = course_details.courseid {
        log::info!("Parsed course {:?}", course_id);
    }

    markdown
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
