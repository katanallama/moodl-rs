// process_results.rs
//
use crate::models::course_content::{Assignment, CourseSection, Grade};
use crate::models::user::User;

#[derive(Debug)]
pub enum ProcessResult {
    User(User),
    Grades(Vec<Grade>),
    Content(Vec<CourseSection>),
    Assigns(Vec<Assignment>),
    None,
}
