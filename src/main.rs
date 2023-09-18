// main.rs
//
mod commands;
mod db;
mod downloader;
mod models;
mod parser;
mod ui;
mod utils;
mod ws;

use crate::{
    commands::{
        command::{Command, DefaultCommand},
        download::DownloadCommand,
        fetch::FetchCommand,
        init::InitCommand,
        parse::ParseCommand,
    },
    db::*,
    models::{configs::*, courses::*},
    ui::prompt::*,
    utils::*,
    ws::*,
};
use eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    setup_logger().expect("Failed to initialize logging");
    initialize_db()?;
    let skin = make_skin();
    let mut config = Configs::new()?;
    let command_enum = prompt_command(&skin)?;
    let client;

    let mut command: Box<dyn Command>;
    command = match command_enum {
        UserCommand::Init => Box::new(InitCommand::new(&mut config, &skin)),
        UserCommand::Fetch => {
            client = ApiClient::from_config(&config)?;
            Box::new(FetchCommand::new(client, &config))
        }
        UserCommand::Parse => Box::new(ParseCommand::new(&config)),
        UserCommand::Download => {
            client = ApiClient::from_config(&config)?;
            Box::new(DownloadCommand::new(client, &config))
        }
        UserCommand::Default => {
            client = ApiClient::from_config(&config)?;
            Box::new(DefaultCommand::new(&config, client))
        }
    };

    command.execute().await?;

    Ok(())
}
