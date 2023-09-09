// models/secrets.rs
//
use crate::{ws::ApiConfig, utils::modify_shortname};
use anyhow::Result;
use {serde::Deserialize, serde::Serialize, std::fs, toml};

#[derive(Debug, Serialize, Deserialize)]
pub struct CourseConfig {
    pub id: i64,
    pub shortname: Option<String>,
    pub filepath: Option<String>,
}

impl From<&crate::Course> for CourseConfig {
    fn from(course: &crate::Course) -> Self {
        if let Some(ref shortname) = course.shortname {
            CourseConfig {
                id: course.id,
                shortname: Some(modify_shortname(&shortname)),
                filepath: None,
                // filepath: Some(format!("./data/{}/", course.shortname)),
            }
        } else {
            CourseConfig {
                id: course.id,
                shortname: course.shortname.clone(),
                filepath: None,
                // filepath: Some(format!("./data/{}/", course.shortname)),
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Secrets {
    pub api: ApiConfig,
    pub courses: Vec<CourseConfig>,
}

impl Secrets {
    pub fn write_courses(&mut self, new_courses: Vec<CourseConfig>) -> Result<()> {
        self.courses = new_courses;
        let updated_secrets = toml::to_string(self)?;
        fs::write("Secrets.toml", updated_secrets)?;

        Ok(())
    }
}

pub fn read_config(path: &str) -> Result<Secrets> {
    let contents = fs::read_to_string(path)?;
    let secrets: Secrets = toml::from_str(&contents)?;
    Ok(secrets)
}
