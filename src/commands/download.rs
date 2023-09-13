// commands/download.rs
//
use {
    crate::commands::command::Command,
    crate::downloader::save_files,
    crate::models::{configs::*, course_details::parse_course_json},
    crate::utils::modify_shortname,
    crate::ws::*,
};
use {async_trait::async_trait, eyre::Result};

pub struct DownloadCommand<'a> {
    client: ApiClient, // owned ApiClient instance
    config: &'a Configs,
}

impl<'a> DownloadCommand<'a> {
    pub fn new(
        client: ApiClient,
        config: &'a Configs,
    ) -> Self {
        Self {
            client,
            config,
        }
    }
}

// TODO LOOK AT THIS
// https://rust-lang.github.io/async-book/07_workarounds/03_send_approximation.html
#[async_trait]
impl<'a> Command for DownloadCommand<'a> {
    async fn execute(&mut self) -> Result<()> {
        self.client = ApiClient::from_config(&self.config)?;

        for course in &self.config.courses {
            let json = parse_course_json(course.id)?;
            if let Some(ref shortname) = course.shortname {
                let file_path = format!("out/{}", modify_shortname(&shortname));
                save_files(&json, &file_path, &self.client).await?;
            } else {
                return Err(eyre::eyre!("No course name found"));
            }
        }

        Ok(())
    }
}
