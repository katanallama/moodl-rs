// handlers.rs
//
use crate::db::insert_assignments;
use crate::db::insert_content;
use crate::db::insert_grades;
use crate::db::insert_user;
use crate::db::insert_pages;
use crate::models::course_content::process_assignments;
use crate::models::course_content::process_content;
use crate::models::course_content::process_grades;
use crate::models::course_content::process_pages;
use crate::models::user::process_courses;
use crate::models::user::process_user;
use crate::models::user::write_course_conf;
use crate::ApiConfig;
use crate::CustomError;
use crate::ProcessResult;

pub async fn store_grades(
    conn: &mut rusqlite::Connection,
    api_config: &mut ApiConfig,
    course_id: i32,
) -> Result<(), CustomError> {
    api_config.courseid = Some(course_id);
    if let ProcessResult::Grades(grades) = api_config
        .call_json(conn, "gradereport_user_get_grade_items", process_grades)
        .await?
    {
        insert_grades(conn, &grades)?;
    }

    Ok(())
}

pub async fn store_content(
    conn: &mut rusqlite::Connection,
    api_config: &mut ApiConfig,
    course_id: i32,
) -> Result<(), CustomError> {
    api_config.userid = None;
    api_config.courseid = Some(course_id);
    if let ProcessResult::Content(cont) = api_config
        .call_json(conn, "core_course_get_contents", process_content)
        .await?
    {
        insert_content(conn, api_config.courseid, &cont)?;
    }

    Ok(())
}

pub async fn store_pages(
    conn: &mut rusqlite::Connection,
    api_config: &mut ApiConfig,
) -> Result<(), CustomError> {
    api_config.courseid = None;
    api_config.userid = None;
    if let ProcessResult::Pages(pages) = api_config
        .call_json(conn, "mod_page_get_pages_by_courses", process_pages)
        .await?
    {
        insert_pages(conn, &pages)?;
    }

    Ok(())
}

pub async fn store_assignments(
    conn: &mut rusqlite::Connection,
    api_config: &mut ApiConfig,
    _course_id: i32,
) -> Result<(), CustomError> {
    api_config.courseid = None;
    api_config.userid = None;
    if let ProcessResult::Assigns(assigns) = api_config
        .call_json(conn, "mod_assign_get_assignments", process_assignments)
        .await?
    {
        insert_assignments(conn, &assigns)?;
    }

    Ok(())
}

pub async fn store_courses(
    conn: &mut rusqlite::Connection,
    api_config: &mut ApiConfig,
) -> Result<(), CustomError> {
    if let ProcessResult::Courses(courses) = api_config
        .call_json(conn, "core_enrol_get_users_courses", process_courses)
        .await?
    {
        write_course_conf(courses)?;
    }

    Ok(())
}

pub async fn store_user(
    conn: &mut rusqlite::Connection,
    api_config: &mut ApiConfig,
) -> Result<(), CustomError> {

    api_config.userid = None;
    if let ProcessResult::User(user) = api_config
        .call_json(conn, "core_webservice_get_site_info", process_user)
        .await?
    {
        insert_user(conn, &user, api_config)?;
    }

    Ok(())
}