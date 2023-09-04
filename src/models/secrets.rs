// models/secrets.rs
//
use crate::ws::ApiConfig;
use anyhow::Result;
use {serde::Deserialize, serde::Serialize, std::fs, toml};

#[derive(Debug, Serialize, Deserialize)]
pub struct CourseSecret {
    pub id: i64,
    pub shortname: Option<String>,
    pub filepath: Option<String>,
}

impl From<&crate::Course> for CourseSecret {
    fn from(course: &crate::Course) -> Self {
        CourseSecret {
            id: course.id,
            shortname: course.shortname.clone(),
            filepath: None,
            // filepath: Some(format!("./data/{}/", course.id)),  // e.g., derive filepath from course id
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Secrets {
    pub api: ApiConfig,
    pub courses: Vec<CourseSecret>,
}

impl Secrets {
    pub fn write_courses(&mut self, new_courses: Vec<CourseSecret>) -> Result<()> {
        self.courses = new_courses;
        let updated_secrets = toml::to_string(self)?;
        fs::write("Secrets.toml", updated_secrets)?;

        Ok(())
    }
}

pub fn read_secrets(path: &str) -> Result<Secrets> {
    let contents = fs::read_to_string(path)?;
    let secrets: Secrets = toml::from_str(&contents)?;
    Ok(secrets)
}
