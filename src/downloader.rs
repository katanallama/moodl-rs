// downloader.rs
//
use crate::{
    db::connect_db, models::configs::Configs, models::course::CourseFile, utils::create_dir,
    ws::ApiClient, utils::home_dir,
};
use eyre::Result;
use log;
use regex::Regex;
use rusqlite::params;

pub async fn save_files(
    api_client: &ApiClient,
    files: Vec<CourseFile>,
    config: &Configs,
) -> Result<()> {
    for file in files {
        let filename = file.filename.unwrap();
        let fileurl = file.fileurl.unwrap();
        let mut file_path = home_dir();

        let course_id = get_course_id(&filename).unwrap().unwrap();

        if let Some(path) = config.get_course_path(course_id) {
            file_path = file_path.join(path);
        }
        if let Some(name) = config.get_course_name(course_id) {
            file_path = file_path.join(name);
        }

        if let Err(e) = handle_file_operations(
            &api_client,
            &file_path.to_str().unwrap(),
            &filename,
            &fileurl,
        )
        .await
        {
            log::error!("Error handling file operations: {:?}", e);
        }
    }

    Ok(())
}

async fn handle_file_operations(
    api_client: &ApiClient,
    file_path: &str,
    filename: &str,
    fileurl: &str,
) -> Result<()> {
    let sanitized_file_name = sanitize_filename(&filename);
    let clean_file_path = format!("{}/{}", file_path, sanitized_file_name);

    match create_dir(&clean_file_path) {
        Ok(_) => match api_client.download_file(&fileurl, &clean_file_path).await {
            Ok(_) => match update_file_paths_in_db(filename, &clean_file_path) {
                Ok(_) => {}
                Err(e) => log::error!("Failed to update DB for Files: {:?}", e),
            },
            Err(e) => log::error!("Failed to download file: {:?}", e),
        },
        Err(e) => log::error!("Failed to create directory: {:?}", e),
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

pub fn update_file_paths_in_db(filename: &str, localpath: &str) -> Result<()> {
    let conn = connect_db()?;
    let sql = format!("UPDATE Files SET localpath = ? WHERE filename = ?");
    conn.execute(sql.as_str(), params![localpath, filename])?;
    Ok(())
}

pub fn get_course_id(filename: &String) -> Result<Option<i64>> {
    let conn = connect_db()?;
    let mut stmt = conn.prepare(
        "
        SELECT
            Sections.courseid
        FROM
            Files
        INNER JOIN
            Modules ON Files.module_id = Modules.moduleid
        INNER JOIN
            Sections ON Modules.section_id = Sections.sectionid
        WHERE
            Files.filename = ?
    ",
    )?;

    let mut rows = stmt.query([filename])?;

    if let Some(row) = rows.next()? {
        Ok(Some(row.get(0)?))
    } else {
        Ok(None)
    }
}
