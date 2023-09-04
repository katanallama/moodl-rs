// models/courses.rs
//
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Courses (pub Vec<Course>);

#[derive(Debug, Serialize, Deserialize)]
pub struct Course {
    pub id: i64,
    pub shortname: Option<String>,
    pub summary: Option<String>,
}

impl<'a> IntoIterator for &'a Courses {
    type Item = &'a Course;
    type IntoIter = std::slice::Iter<'a, Course>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
