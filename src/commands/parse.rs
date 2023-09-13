// commands/parse.rs
//
use {
    crate::commands::command::Command,
    crate::models::{configs::*, course_details::parse_course_json},
    crate::parser::save_markdown_to_file,
    crate::utils::modify_shortname,
};
use {async_trait::async_trait, eyre::Result};

pub struct ParseCommand<'a> {
    config: &'a Configs,
}

impl<'a> ParseCommand<'a> {
    pub fn new(config: &'a Configs) -> Self {
        Self { config }
    }
}

#[async_trait]
impl<'a> Command for ParseCommand<'a> {
    async fn execute(&mut self) -> Result<()> {
        parse_command_handler(self.config)?;
        Ok(())
    }
}

pub fn parse_command_handler(config: &Configs) -> Result<()> {
    for course in &config.courses {
        let json = parse_course_json(course.id)?;
        if let Some(ref shortname) = course.shortname {
            let file_path = format!("out/{}", modify_shortname(&shortname));
            save_markdown_to_file(&json, &file_path)?;
        } else {
            return Err(eyre::eyre!("No course name found"));
        }
    }
    Ok(())
}
