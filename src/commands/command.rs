// commands/command.rs
//
use super::{download::DownloadCommand, fetch::FetchCommand, parse::ParseCommand};
use crate::{models::configs::Configs, ws::ApiClient};
use {async_trait::async_trait, eyre::Result};

#[async_trait]
pub trait Command {
    async fn execute(&mut self) -> Result<()>;
}

pub struct DefaultCommand<'a> {
    config: &'a Configs,
    client: ApiClient,
}

impl<'a> DefaultCommand<'a> {
    pub fn new(config: &'a Configs, client: ApiClient) -> Self {
        Self { config, client }
    }
}

#[async_trait]
impl<'a> Command for DefaultCommand<'a> {
    async fn execute(&mut self) -> Result<()> {
        let mut fetch_command = FetchCommand::new(self.client.clone(), self.config);
        fetch_command.execute().await?;

        let mut download_command = DownloadCommand::new(self.client.clone(), self.config);
        download_command.execute().await?;

        let mut parse_command = ParseCommand::new(self.config);
        parse_command.execute().await?;

        Ok(())
    }
}
