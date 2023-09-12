// models/configs.rs
//
use crate::utils::{config_dir, create_dir, modify_shortname};
use {
    config::{Config, File},
    eyre::{Result, WrapErr},
    serde::{Deserialize, Serialize},
    std::{fs, io, path::Path},
    termimad::{MadSkin, Question},
    toml,
};

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
        init_config_file()?;

        let config_path = config_dir().join("config.toml");

        let s = Config::builder()
            .add_source(File::from(config_path))
            .build()?;

        Ok(s.try_deserialize()?)
    }

    pub fn write_to_file(&mut self) -> Result<()> {
        let data = toml::to_string(self).wrap_err("Failed to serialize config to TOML format")?;
        let config_file = config_dir().join("config.toml");
        fs::write(config_file, data).wrap_err("Failed to write updated config to file")
    }

    pub fn write_baseurl(&mut self, baseurl: &String) -> Result<()> {
        self.api.base_url = baseurl.to_string();
        log::info!("Wrote base url {} to 'config.toml'", baseurl);
        self.write_to_file()
    }

    pub fn write_courses(&mut self, new_courses: Vec<CourseConfig>) -> Result<()> {
        self.courses = new_courses;
        log::info!("Wrote courses to 'config.toml'");
        self.write_to_file()
    }

    pub fn write_token(&mut self, token: &String) -> Result<()> {
        self.api.token = token.to_string();
        log::info!("Wrote token {} to 'config.toml'", token);
        self.write_to_file()
    }

    pub fn write_userid(&mut self, userid: i64) -> Result<()> {
        self.api.userid = userid;
        log::info!("Wrote user id {} to 'config.toml'", userid);
        self.write_to_file()
    }

    pub async fn prompt_config(&mut self, skin: &MadSkin) -> Result<()> {
        let question = format!(
            "Would you like to configure your moodle url and API token now?
    You will only have to do this once.",
        );

        let mut q = Question::new(&question);
        q.add_answer('y', "**Y**es, configure now");
        q.add_answer('n', "**N**o, skip and configure manually");
        q.set_default('y');

        let answer = q.ask(skin)?;

        if answer == "y" {
            self.write_baseurl(&Self::prompt_user_url().wrap_err("Invalid URL")?)?;
            self.write_token(&Self::prompt_user_token().wrap_err("Invalid token")?)?;
        }
        Ok(())
    }

    pub fn prompt_user_url() -> Result<String> {
        println!("Please enter your Moodle URL: ");
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(input.trim().to_string())
    }

    pub fn prompt_user_token() -> Result<String> {
        println!("Please enter your API token: ");
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(input.trim().to_string())
    }
}

pub fn init_config_file() -> Result<(), eyre::Report> {
    let binding = config_dir().join("config.toml");
    let dest_path = binding
        .to_str()
        .ok_or_else(|| eyre::eyre!("Path conversion to string failed"))?;
    create_dir(dest_path).wrap_err("Failed to create config directory")?;

    if !Path::new(&dest_path).exists() {
        let config_template = include_str!("../config.toml");
        fs::write(&dest_path, config_template)
            .wrap_err("Failed to write example config to user's config directory")?;
        log::info!("Created example config at {}", dest_path);
    }

    Ok(())
}
