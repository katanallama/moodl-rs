// commands.rs
//
use async_trait::async_trait;
use {
    crate::commands::command::Command,
    crate::db::connect_db,
    crate::models::{configs::*, course_section::insert_sections, pages::*},
    crate::ws::*,
};
use {eyre::Result, rusqlite::Connection};

pub struct FetchCommand<'a> {
    client: ApiClient, // owned ApiClient instance
    config: &'a Configs,
}

impl<'a> FetchCommand<'a> {
    pub fn new(client: ApiClient, config: &'a Configs) -> Self {
        Self { client, config }
    }
}

#[async_trait]
impl<'a> Command for FetchCommand<'a> {
    async fn execute(&mut self) -> Result<()> {
        let mut conn = connect_db()?;
        self.client = ApiClient::from_config(&self.config)?;
        fetch_course_handler(&mut self.client, &mut conn, &self.config).await?;
        fetch_page_handler(&mut self.client, &mut conn).await?;

        Ok(())
    }
}

pub async fn fetch_page_handler(client: &ApiClient, conn: &mut Connection) -> Result<()> {
    let mut response = client.fetch_course_pages().await?;
    if let ApiResponse::Pages(ref mut pages) = response {
        insert_pages(conn, &mut pages.pages)?;
    } else {
        return Err(eyre::eyre!("Unexpected API response: {:?}", response));
    }
    Ok(())
}

pub async fn fetch_course_handler(
    client: &mut ApiClient,
    conn: &mut Connection,
    config: &Configs,
) -> Result<()> {
    for course in &config.courses {
        let response = client.fetch_course_contents(course.id).await?;
        if let ApiResponse::Sections(mut sections) = response {
            insert_sections(conn, &mut sections, course.id)?;
        } else {
            return Err(eyre::eyre!("Unexpected API response: {:?}", response));
        }
    }
    Ok(())
}
