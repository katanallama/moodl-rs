// commands/download.rs
//
use crate:: {
    commands::command::Command,
    db::connect_db,
    downloader::save_files,
    models::configs::*,
    models::course::get_all_files,
    ws::*,
};
use {async_trait::async_trait, eyre::Result};

pub struct DownloadCommand<'a> {
    client: ApiClient, // owned ApiClient instance
    config: &'a Configs,
}

impl<'a> DownloadCommand<'a> {
    pub fn new(client: ApiClient, config: &'a Configs) -> Self {
        Self { client, config }
    }
}

#[async_trait]
impl<'a> Command for DownloadCommand<'a> {
    async fn execute(&mut self) -> Result<()> {
        self.client = ApiClient::from_config(&self.config)?;
        let conn = connect_db();
        let files = get_all_files(&mut conn.unwrap())?;
        save_files(&self.client, files, &self.config).await?;

        Ok(())
    }
}
