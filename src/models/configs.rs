// models/configs.rs
//
use crate::utils::modify_shortname;
use anyhow::Result;
use config::{Config, File};
use {serde::Deserialize, serde::Serialize, std::fs, toml};

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
            }
        } else {
            CourseConfig {
                id: course.id,
                shortname: course.shortname.clone(),
                filepath: None,
            }
        }
    }
}

impl Configs {
    pub fn new() -> Result<Self> {

        let s = Config::builder()
            // merge in the "Configs" configuration file 'src/config.toml'
            .add_source(File::with_name("src/config"))
            .build()?;

        println!("api - base_url: {:?}", s.get::<String>("api.base_url"));
        println!("api - token: {:?}", s.get::<String>("api.token"));
        println!("api - userid: {:?}", s.get::<String>("api.userid"));

        // deserialize (and thus freeze) the entire configuration as
        Ok(s.try_deserialize()?)
    }

    pub fn write_userid(&mut self, userid: i64) -> Result<()> {
        self.api.userid = userid;
        let updated_config = toml::to_string(self)?;
        fs::write("src/config.toml", updated_config)?;

        Ok(())
    }

    pub fn write_courses(&mut self, new_courses: Vec<CourseConfig>) -> Result<()> {
        self.courses = new_courses;
        let updated_courses = toml::to_string(self)?;
        fs::write("src/config.toml", updated_courses)?;

        Ok(())
    }
}

pub fn read_config(path: &str) -> Result<Configs> {
    let contents = fs::read_to_string(path)?;
    let configs: Configs = toml::from_str(&contents)?;
    Ok(configs)
}
