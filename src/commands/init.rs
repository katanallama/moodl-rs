// commands/init.rs
//
use async_trait::async_trait;
use {
    crate::commands::command::Command,
    crate::db::{connect_db, create_tables},
    crate::models::configs::*,
    crate::prompt_courses,
    crate::ws::*,
};
use {eyre::Result, termimad::MadSkin};

pub struct InitCommand<'a> {
    config: &'a mut Configs,
    skin: &'a MadSkin,
}

impl<'a> InitCommand<'a> {
    pub fn new(config: &'a mut Configs, skin: &'a MadSkin) -> Self {
        Self { config, skin }
    }
}

#[async_trait]
impl<'a> Command for InitCommand<'a> {
    async fn execute(&mut self) -> Result<()> {
        let conn = connect_db()?;
        create_tables(&conn)?;

        self.config.prompt_config(&self.skin).await?;

        let mut client = ApiClient::from_config(&self.config)?;
        let user_id = get_user_id(&mut client).await?;
        self.config.write_userid(user_id)?;

        let mut client = ApiClient::from_config(&self.config)?;
        fetch_course_ids_handler(&self.skin, &mut client, &mut self.config).await?;

        Ok(())
    }
}

pub async fn get_user_id(client: &mut ApiClient) -> Result<i64> {
    let response = client.fetch_user_id().await?;
    if let ApiResponse::SiteInfo(info) = response {
        return Ok(info.userid);
    } else {
        return Err(eyre::eyre!("Unexpected API response: {:?}", response));
    }
}

pub async fn fetch_course_ids_handler(
    skin: &MadSkin,
    client: &mut ApiClient,
    config: &mut Configs,
) -> Result<()> {
    let response = client.fetch_user_courses().await?;
    if let ApiResponse::Course(course_list) = response {
        let selected_courses = prompt_courses(&course_list, &skin)?;
        config.write_courses(selected_courses)?;
    } else {
        return Err(eyre::eyre!("Unexpected API response: {:?}", response));
    }

    Ok(())
}
