use std::string::String;

pub struct SaveEvent;
pub struct LoadEvent;
pub struct CloseProjectEvent;
pub struct OpenProjectEvent{
    pub project_path: String,
}
pub struct CreateProjectEvent{
    pub project_path: String,
}