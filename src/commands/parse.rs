// commands/parse.rs
//
use {
    crate::commands::command::Command,
    crate::db::connect_db,
    crate::parser::parse,
    crate::utils::home_dir,
    crate::models::configs::*,
    crate::models::course::retrieve_course_structure,
    crate::parser::save_markdown_to_file,
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
        let conn = connect_db();
        let structure = retrieve_course_structure(&mut conn.unwrap(), course.id)?;
        let course_md = parse(structure);

        let mut file_path = home_dir();
        if let Some(course_path) = config.get_course_path(course.id){
            file_path = file_path.join(course_path);
        }
        if let Some(name) = config.get_course_name(course.id) {
            file_path = file_path.join(name);
        }
        save_markdown_to_file(course_md, file_path.to_str().unwrap())?;

    }
    Ok(())
}
