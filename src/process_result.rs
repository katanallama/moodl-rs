// process_results.rs
//
use crate::models::course_content::{Assignment, CourseSection, Grade, Page};
use crate::models::user::{User, Course};

#[derive(Debug)]
pub enum ProcessResult {
    User(User),
    Courses(Vec<Course>),
    Grades(Vec<Grade>),
    Content(Vec<CourseSection>),
    Assigns(Vec<Assignment>),
    Pages(Vec<Page>),
    None,
}
