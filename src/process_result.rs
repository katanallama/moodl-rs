// process_results.rs
//
use crate::models::course::Course;
use crate::models::course_grades::Table;
use crate::models::course_content::CourseSection;
use crate::models::course_content::Assignment;

#[derive(Debug)]
pub enum ProcessResult {
    UserId(i32),
    Courses(Vec<Course>),
    Grades(Vec<Table>),
    Content(Vec<CourseSection>),
    Assigns(Vec<Assignment>),
    None,
}
