use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;
use bevy_ecs::prelude::ReflectComponent;
use serde::{
    Serialize,
    Deserialize,
};
use egui::Ui;
use std::sync::Arc;
use std::path::PathBuf;

pub fn default_ui() -> Option<Ui> {
    None
}

#[derive(Component, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct MainMenuComponent{
    #[serde(skip, default="default_ui")]
    #[reflect(ignore)]
    pub ui: Option<Ui>
}

impl Default for MainMenuComponent{
    fn default() -> Self{
        MainMenuComponent{
            ui: None,
        }
    }
}

#[derive(Component, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct FileSubMenuComponent{
    pub parent: Option<Entity>,
    pub new_project_window: bool,
    pub open_project_window: bool,
    #[reflect(ignore)]
    pub current_nav_path: PathBuf,
    #[reflect(ignore)]
    pub text_entry: String,
}

impl Default for FileSubMenuComponent {
    fn default() -> Self {
        FileSubMenuComponent{
            parent: None,
            new_project_window: false,
            open_project_window: false,
            current_nav_path: std::env::current_dir().unwrap(),
            text_entry: std::string::String::from(""),
        }
    }
}

impl FileSubMenuComponent{
    pub fn new() -> Self {
        FileSubMenuComponent{
            parent: None,
            new_project_window: false,
            open_project_window: false,
            current_nav_path: std::env::current_dir().unwrap(),
            text_entry: std::string::String::from(""),
        }
    }

    pub fn with_parent(entity: Entity) -> Self{
        FileSubMenuComponent{
            parent: Some(entity),
            new_project_window: false,
            open_project_window: false,
            current_nav_path: std::env::current_dir().unwrap(),
            text_entry: std::string::String::from(""),
        }
    }
}

#[derive(Component, Reflect, Clone, Serialize, Deserialize)]
pub struct FileMenuSaveComponent;