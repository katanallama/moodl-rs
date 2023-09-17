use crate::models::course::CourseSection;
use eyre::Result;
use html2md::parse_html;
use std::{fs::File as StdFile, io::Write};

pub fn parse(course: Vec<CourseSection>) -> String {
    let mut markdown = String::new();

    course.into_iter().for_each(|section| {
        log::info!("Section name: {}", section.name);

        markdown.push_str(&format!("# {}\n", section.name));
        let summary = clean_html(&section.summary);
        markdown.push_str(&format!("{}\n\n", parse_html(&summary)));

        let mut is_first_module = true;

        section.modules.into_iter().for_each(|module| {
            // log::info!("Module name: {}", module.name);

            if is_first_module {
                markdown.push_str(&format!("## {}\n", module.name));
                is_first_module = false;
            } else {
                markdown.push_str(&format!("### {}\n", module.name));
            }

            if let Some(desc) = &module.description {
                let desc = clean_html(&desc);
                if desc.trim() != module.name.trim() {
                    markdown.push_str(&format!("{}\n\n", parse_html(&desc)));
                }
            }

            match module.contents {
                Some(files) => {
                    files.into_iter().for_each(|file| {
                        if let Some(name) = &file.filename {
                            if let Some(path) = &file.filepath {
                                // log::info!("Module file: {} at {}", name, path);
                                markdown.push_str(&format!("\n[{}]", name));
                                markdown.push_str(&format!("({})\n\n", path));
                            } else if let Some(url) = &file.fileurl {
                                markdown.push_str(&format!("\n[{}]", name));
                                markdown.push_str(&format!("({})\n\n", url));
                            }
                        }
                    });
                }
                _ => (),
            }
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

    let mut clean_html = html.replace("<h5></h5>", "");
    clean_html = clean_html.replace(" dir=\"ltr\" style=\"text-align: left;\"", "");
    clean_html = clean_html.replace("<h5>", "");
    clean_html = clean_html.replace("</h5>", "");
    clean_html = clean_html.replace("<br>", "");
    clean_html = clean_html.replace("</ br>", "");
    clean_html = clean_html.replace("<br />", "");
    clean_html = clean_html.replace("\r", "");


    clean_html = clean_html.replace("<h4></h4>", "");
    clean_html = clean_html.replace("<p></p>", "");
    clean_html = clean_html.replace("<b></b>", "");

    clean_html.to_string()
}
