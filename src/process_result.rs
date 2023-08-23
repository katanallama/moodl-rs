// process_results.rs
//
use crate::models::course::Course;
use crate::models::course_grades::Table;

#[derive(Debug)]
pub enum ProcessResult {
    UserId(i32),
    Courses(Vec<Course>),
    Grades(Vec<Table>),
    None,
}
