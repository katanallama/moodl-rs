// models/secrets.rs
//
use crate::ws::ApiConfig;
use anyhow::Result;
use {serde::Deserialize, serde::Serialize, std::fs, toml};

// use super::courses::Courses;

#[derive(Debug, Serialize, Deserialize)]
pub struct CourseSecret {
    pub id: i32,
    pub shortname: Option<String>,
    pub filepath: Option<String>,  // new field
}

impl From<&crate::Course> for CourseSecret {
    fn from(course: &crate::Course) -> Self {
        CourseSecret {
            id: course.id,
            shortname: course.shortname.clone(),
            filepath: None,
            // filepath: Some(format!("./data/{}.txt", course.id)),  // e.g., derive filepath from course id
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Secrets {
    pub api: ApiConfig,
    pub courses: Vec<CourseSecret>,
}

impl Secrets {
    pub fn write_courses(&mut self, new_courses: crate::ApiResponse) -> Result<()> {
        // Convert the courses from the ApiResponse into CourseSecrets
        if let crate::ApiResponse::Course(course_list) = &new_courses {
            let course_secrets: Vec<CourseSecret> = course_list.iter().map(CourseSecret::from).collect();

            // Update the courses in the current state
            self.courses = course_secrets;

            // Serialize the updated data into TOML format
            let updated_secrets = toml::to_string(self)?;

            // Write the serialized data back to the 'Secrets.toml' file
            fs::write("Secrets.toml", updated_secrets)?;
        } else {
            // Handle unexpected `ApiResponse` variants, possibly return an error or log
            // ...
        }

        Ok(())
    }
}

pub fn read_secrets(path: &str) -> Result<Secrets> {
    let contents = fs::read_to_string(path)?;
    let secrets: Secrets = toml::from_str(&contents)?;
    Ok(secrets)
}
