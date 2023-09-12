// handlers.rs
//
use {
    crate::downloader::save_files,
    crate::models::{
        configs::*, course_details::parse_course_json, course_section::insert_sections, pages::*,
    },
    crate::parser::save_markdown_to_file,
    crate::prompt_courses,
    crate::utils::modify_shortname,
    crate::ws::*,
};
use {eyre::Result, rusqlite::Connection, termimad::MadSkin};

pub async fn get_userid(client: &mut ApiClient) -> Result<i64> {
    let response = client.fetch_user_id().await?;
    if let ApiResponse::SiteInfo(info) = response {
        return Ok(info.userid);
    } else {
        return Err(eyre::eyre!("Unexpected API response: {:?}", response));
    }
}

pub async fn get_courses(
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

pub async fn fetch_page_handler(
    // _config: Configs,
    client: &mut ApiClient,
    conn: &mut Connection,
) -> Result<()> {
    let mut response = client.fetch_course_pages().await?;
    if let ApiResponse::Pages(ref mut pages) = response {
        insert_pages(conn, &mut pages.pages)?;
    } else {
        return Err(eyre::eyre!("Unexpected API response: {:?}", response));
    }
    Ok(())
}

pub async fn fetch_course_handler(
    config: Configs,
    client: &mut ApiClient,
    conn: &mut Connection,
) -> Result<()> {
    for course in config.courses {
        let response = client.fetch_course_contents(course.id).await?;
        if let ApiResponse::Sections(mut sections) = response {
            insert_sections(conn, &mut sections, course.id)?;
        } else {
            return Err(eyre::eyre!("Unexpected API response: {:?}", response));
        }
    }
    Ok(())
}

pub async fn parse_command_handler(config: Configs, conn: &Connection) -> Result<()> {
    for course in config.courses {
        let json = parse_course_json(&conn, course.id)?;
        if let Some(ref shortname) = course.shortname {
            let file_path = format!("out/{}", modify_shortname(&shortname));
            save_markdown_to_file(&json, &file_path)?;
        }
        else {
            return Err(eyre::eyre!("No course name found"));
        }
    }
    Ok(())
}

pub async fn download_command_handler(
    config: Configs,
    client: &ApiClient,
    conn: &Connection,
) -> Result<()> {
    for course in config.courses {
        let json = parse_course_json(&conn, course.id)?;
        if let Some(ref shortname) = course.shortname {
            let file_path = format!("out/{}", modify_shortname(&shortname));
            save_files(&json, &file_path, &client, &conn).await?;
        } else {
            return Err(eyre::eyre!("No course name found"));
        }
    }
    Ok(())
}
