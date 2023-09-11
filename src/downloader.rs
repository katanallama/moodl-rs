// downloader.rs
use crate::{
    models::course_details::{GetFileData, ParseCourseDetails},
    utils::create_dir,
    ws::ApiClient,
};
use eyre::Result;
use log;
use regex::Regex;
use rusqlite::{params, Connection};
use serde_json;

pub async fn save_files(
    json_data: &str,
    file_path: &str,
    api_client: &ApiClient,
    conn: &Connection,
) -> Result<()> {
    let parsed_course_details: ParseCourseDetails = serde_json::from_str(json_data)?;

    for section in parsed_course_details.sections {
        for module in section.modules {
            for content in &module.content {
                if let Some((filename, fileurl)) = content.get_file_data() {
                    handle_file_operations(
                        api_client,
                        conn,
                        file_path,
                        "Content",
                        content.content_id,
                        filename,
                        fileurl,
                    )
                    .await?;
                }
            }

            for page in &module.pages {
                for file in &page.files {
                    if let Some((filename, fileurl)) = file.get_file_data() {
                        handle_file_operations(
                            api_client,
                            conn,
                            file_path,
                            "Files",
                            file.file_id,
                            filename,
                            fileurl,
                        )
                        .await?;
                    }
                }
            }
        }
    }

    Ok(())
}

async fn handle_file_operations(
    api_client: &ApiClient,
    conn: &Connection,
    file_path: &str,
    table_name: &str,
    id_option: Option<i64>,
    filename: String,
    fileurl: String,
) -> Result<()> {
    if let Some(id) = id_option {
        let sanitized_file_name = sanitize_filename(&filename);
        let clean_file_path = format!("{}/{}", file_path, sanitized_file_name);

        match create_dir(&clean_file_path) {
            Ok(_) => match api_client.download_file(&fileurl, &clean_file_path).await {
                Ok(_) => {
                    match update_file_paths_in_db(conn, table_name, "id", id, &clean_file_path) {
                        Ok(_) => {}
                        Err(e) => log::error!("Failed to update DB for {}: {:?}", table_name, e),
                    }
                }
                Err(e) => log::error!("Failed to download file: {:?}", e),
            },
            Err(e) => log::error!("Failed to create directory: {:?}", e),
        }
    } else {
        log::error!("ID not found for file: {}", filename);
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
