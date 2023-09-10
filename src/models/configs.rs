// models/configs.rs
//
use crate::utils::modify_shortname;
use config::{Config, File};
use eyre::{Result, WrapErr};
use serde::{Deserialize, Serialize};
use std::fs;
use toml;

#[derive(Debug, Serialize, Deserialize)]
pub struct Configs {
    pub api: ApiConfig,
    pub courses: Vec<CourseConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiConfig {
    pub base_url: String,
    pub token: String,
    pub userid: i64,
}

impl ApiConfig {
    pub fn new(base_url: String, token: String, userid: i64) -> Self {
        Self {
            base_url,
            token,
            userid,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CourseConfig {
    pub id: i64,
    pub shortname: Option<String>,
}

impl CourseConfig {
    pub fn new(id: i64, shortname: Option<String>) -> Self {
        Self { id, shortname }
    }
}

impl From<&crate::Course> for CourseConfig {
    fn from(course: &crate::Course) -> Self {
        if let Some(ref shortname) = course.shortname {
            CourseConfig {
                id: course.id,
                shortname: Some(modify_shortname(&shortname)),
            }
        } else {
            CourseConfig {
                id: course.id,
                shortname: course.shortname.clone(),
            }
        }
    }
}

impl Configs {
    pub fn new() -> Result<Self> {
        let s = Config::builder()
            .add_source(File::with_name("src/config"))
            .build()?;

        log::info!(
            "\napi - base_url: {:?} \napi - token: {:?}\napi - userid: {:?}",
            s.get::<String>("api.base_url"),
            s.get::<String>("api.token"),
            s.get::<String>("api.userid")
        );

        Ok(s.try_deserialize()?)
    }

    pub fn write_to_file(&mut self) -> Result<()> {
        let data = toml::to_string(self).wrap_err("Failed to serialize config to TOML format")?;
        fs::write("src/config.toml", data).wrap_err("Failed to write updated config to file")
    }

    pub fn write_userid(&mut self, userid: i64) -> Result<()> {
        self.api.userid = userid;
        log::info!("Wrote user id {} to 'config.toml'", userid);
        self.write_to_file()
    }

    pub fn write_courses(&mut self, new_courses: Vec<CourseConfig>) -> Result<()> {
        self.courses = new_courses;
        log::info!("Wrote courses to 'config.toml'");
        self.write_to_file()
    }
}

pub fn read_config(path: &str) -> Result<Configs> {
    let contents = fs::read_to_string(path).wrap_err("Failed to read config file from path")?;
    let configs: Configs = toml::from_str(&contents).wrap_err("Failed to parse config file")?;
    Ok(configs)
}
