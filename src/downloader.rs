// downloader.rs
use crate::models::course_details::ParseCourseDetails;
use crate::utils::home_dir;
use crate::ws::ApiClient;
use anyhow::Result;
use regex::Regex;
use rusqlite::{params, Connection};
use serde_json;
use std::fs;
use std::path::Path;

pub async fn save_files(
    json_data: &str,
    file_path: &str,
    api_client: &ApiClient,
    conn: &Connection,
) -> Result<()> {
    let parsed_course_details: ParseCourseDetails = serde_json::from_str(json_data)?;
    // parse_files(parsed_course_details, api_client, &file_path).await?;
    for section in parsed_course_details.sections {
        for module in section.modules {
            for content in &module.content {
                if let Some(content_filename) = &content.content_filename {
                    if let Some(content_fileurl) = &content.content_fileurl {
                        let sanitized_file_name = sanitize_filename(content_filename);
                        let clean_file_path = format!("{}/{}", file_path, sanitized_file_name);

                        create_directory_if_not_exists(&clean_file_path)?;
                        api_client
                            .download_file(content_fileurl, &clean_file_path)
                            .await;
                            // .await?;

                        update_file_paths_in_db(
                            &conn,
                            "Content",
                            "id",
                            content.content_id.unwrap(),
                            &clean_file_path,
                        )?;
                    }
                }
            }

            for page in &module.pages {
                for file in &page.files {
                    if let Some(file_filename) = &file.file_filename {
                        if let Some(file_fileurl) = &file.file_fileurl {
                            let sanitized_file_name = sanitize_filename(file_filename);
                            let clean_file_path = format!("{}/{}", file_path, sanitized_file_name);

                            create_directory_if_not_exists(&clean_file_path)?;
                            api_client
                                .download_file(file_fileurl, &clean_file_path)
                                .await;
                                // .await?;

                            update_file_paths_in_db(
                                &conn,
                                "Files",
                                "id",
                                file.file_id.unwrap(),
                                &clean_file_path,
                            )?;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn create_directory_if_not_exists(file_path: &str) -> Result<()> {
    let path = Path::new(file_path);
    if let Some(parent_path) = path.parent() {
        if !parent_path.exists() {
            fs::create_dir_all(parent_path)?;
        }
    }
    Ok(())
}

fn sanitize_filename(filename: &str) -> String {
    let re = Regex::new(r"[^\w\.\-]").unwrap();
    let intermediate = re.replace_all(filename, "");
    let whitespace_and_underscores = Regex::new(r"[\s_]+").unwrap();
    whitespace_and_underscores
        .replace_all(&intermediate, "-")
        .to_string()
}

pub fn update_file_paths_in_db(
    conn: &Connection,
    table_name: &str,
    id_column_name: &str,
    id: i64,
    localpath: &str,
) -> Result<()> {
    let sql = format!(
        "UPDATE {} SET localpath = ? WHERE {} = ?",
        table_name, id_column_name
    );
    conn.execute(sql.as_str(), params![localpath, id])?;
    Ok(())
}
