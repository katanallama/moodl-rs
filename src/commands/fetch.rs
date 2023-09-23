use crate::models::{assignments::insert_assignments, scorm::insert_scorms};

// commands.rs
//
use {
    crate::commands::command::Command,
    crate::db::connect_db,
    crate::models::{
        configs::*,
        course::{insert_course_sections, Pages},
        grades::insert_grades,
    },
    crate::ws::*,
};
use {async_trait::async_trait, eyre::Result, rusqlite::Connection};

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

        let pages = fetch_page_handler(&mut self.client).await?;
        fetch_course_handler(&mut self.client, &mut conn, &self.config, pages).await?;
        fetch_assignment_handler(&mut self.client).await?;
        fetch_grade_handler(&mut self.client, &self.config).await?;
        fetch_scorm_handler(&mut self.client).await?;

        Ok(())
    }
}

pub async fn fetch_assignment_handler(client: &ApiClient) -> Result<()> {
    let mut conn = connect_db()?;
    let response = client.fetch_assignments().await?;
    if let ApiResponse::Assignments(assignments) = response {
        log::debug!("{:#?}", assignments);
        insert_assignments(&mut conn, assignments)?;
    } else {
        return Err(eyre::eyre!("Unexpected API response: {:?}", response));
    }
    Ok(())
}

pub async fn fetch_grade_handler(client: &ApiClient, config: &Configs) -> Result<()> {
    for course in &config.courses {
        let mut conn = connect_db()?;
        let response = client.fetch_course_grades(course.id).await?;
        if let ApiResponse::UserGrades(grades) = response {
            log::debug!("{:#?}", grades);
            insert_grades(&mut conn, grades.usergrades)?;
        } else {
            return Err(eyre::eyre!("Unexpected API response: {:?}", response));
        }
    }
    Ok(())
}

pub async fn fetch_page_handler(client: &ApiClient) -> Result<Pages> {
    let response = client.fetch_course_pages().await?;
    if let ApiResponse::Pages(pages) = response {
        Ok(pages)
    } else {
        return Err(eyre::eyre!("Unexpected API response: {:?}", response));
    }
}

pub async fn fetch_course_handler(
    client: &mut ApiClient,
    conn: &mut Connection,
    config: &Configs,
    pages: Pages,
) -> Result<()> {
    for course in &config.courses {
        let response = client.fetch_course_contents(course.id).await?;
        if let ApiResponse::Sections(mut sections) = response {
            log::debug!("{:#?}", sections);
            insert_course_sections(conn, &mut sections, &pages, course.id)?;
        } else {
            return Err(eyre::eyre!("Unexpected API response: {:?}", response));
        }
    }
    Ok(())
}

pub async fn fetch_scorm_handler(client: &ApiClient) -> Result<()> {
    let response = client.fetch_scorms().await?;
    if let ApiResponse::Scorms(scorms) = response {
        log::debug!("{:#?}", scorms);
        insert_scorms(&mut connect_db()?, scorms)?;
    } else {
        return Err(eyre::eyre!("Unexpected API response: {:?}", response));
    }
    Ok(())
}
