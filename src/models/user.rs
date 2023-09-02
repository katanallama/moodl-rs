// models/user.rs
//
use {
    crate::db::*,
    crate::models::response::CustomError,
    crate::process_result::ProcessResult,
    crate::ws::ApiConfig,
    chrono::Utc,
    rusqlite::{Connection, Result},
    serde::{Deserialize, Serialize},
    serde_json::Value,
    std::fs,
};

#[derive(Debug, Deserialize, Serialize)]
struct Secrets {
    api: ApiSecrets,
    courses: Vec<CourseSecrets>,
}

#[derive(Debug, Deserialize, Serialize)]
struct CourseSecrets {
    id: i64,
    shortname: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ApiSecrets {
    url: String,
    token: String,
    private_key: String,
}

#[derive(Deserialize, Debug)]
pub struct User {
    pub id: i64,
    pub content: String,
    pub privkey: String,
    pub url: String,
    pub wstoken: String,
    pub lastfetched: i64,
}

#[derive(Deserialize, Debug)]
pub struct Course {
    pub id: i64,
    pub courseid: Option<i64>,
    pub shortname: String,
    pub fullname: String,
    pub lastfetched: i64,
}

fn read_secrets() -> Result<Secrets, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string("Secrets.toml")?;
    let secrets: Secrets = toml::from_str(&contents)?;
    Ok(secrets)
}

pub fn write_course_conf(courses: Vec<Course>) -> Result<ProcessResult, CustomError> {
    let contents = fs::read_to_string("Secrets.toml")?;
    let mut secrets: Secrets = toml::from_str(&contents)?;

    secrets.courses.clear(); // Clear existing courses

    for course in &courses {
        if let Some(course_id) = course.courseid {
            let decoded_shortname = serde_json::from_str::<String>(&course.shortname)
                .unwrap_or_else(|_| course.shortname.clone());
            secrets.courses.push(CourseSecrets {
                id: course_id,
                shortname: decoded_shortname,
            });
        }
    }

    // Serialize and write back to file
    let updated_secrets = toml::to_string(&secrets)?;
    fs::write("Secrets.toml", updated_secrets)?;

    Ok(ProcessResult::Courses(courses))
}

pub fn process_courses(_conn: &Connection, content: &str) -> Result<ProcessResult, CustomError> {
    let parsed: Value = serde_json::from_str(content)?;

    let mut course_list: Vec<Course> = Vec::new();
    let now = Utc::now().timestamp();

    if let Some(courses) = parsed.as_array() {
        for course in courses {
            let course = Course {
                id: 0,
                courseid: course["id"].as_i64(),
                shortname: course["shortname"].to_string(),
                fullname: course["fullname"].to_string(),
                lastfetched: now,
            };
            course_list.push(course);
        }
    }

    Ok(ProcessResult::Courses(course_list))
}

pub fn process_user(_conn: &Connection, content: &str) -> Result<ProcessResult, CustomError> {
    let parsed: Value = serde_json::from_str(content)?;
    let now = Utc::now().timestamp();
    let secrets = read_secrets()?;

    if let Some(user_id) = parsed["userid"].as_i64() {
        let user = User {
            id: user_id,
            content: parsed.to_string(),
            privkey: secrets.api.private_key.clone(),
            url: secrets.api.url.clone(),
            wstoken: secrets.api.token.clone(),
            lastfetched: now,
        };

        return Ok(ProcessResult::User(user));
    }

    Ok(ProcessResult::None)
}

pub async fn store_user(
    conn: &mut rusqlite::Connection,
    api_config: &mut ApiConfig,
) -> Result<User, CustomError> {
    api_config.userid = None;
    let user = if let ProcessResult::User(user) = api_config
        .call_json(conn, "core_webservice_get_site_info", process_user)
        .await?
    {
        generic_insert(conn, &user)?;
        Some(user)
    } else {
        None
    };

    user.ok_or(CustomError::Other("Failed to store user".to_string()))
}

pub async fn init(conn: &mut rusqlite::Connection) -> Result<ApiConfig, CustomError> {
    let secrets = read_secrets()?;

    let wstoken = secrets.api.token;
    if wstoken.is_empty() {
        return Err(CustomError::Other("Moodle key is required!".to_string()));
    }

    let url = if secrets.api.url.trim().is_empty() {
        "https://urcourses.uregina.ca/webservice/rest/server.php".to_string()
    } else {
        secrets.api.url
    };

    let mut api_config = ApiConfig {
        wstoken,
        courseid: None,
        userid: None,
        client: reqwest::Client::new(),
        url,
    };

    let user = store_user(conn, &mut api_config).await?;

    api_config.userid = Some(user.id);

    Ok(api_config)
}
