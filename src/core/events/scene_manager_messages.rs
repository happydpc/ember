use std::string::String;

#[derive(Clone)]
pub enum SceneManagerMessage{
    OpenProject{
        path: String,
        scene_name: String,
    },
    CloseProject,

}