use bevy_ecs::component::Component;


use bevy_reflect::Reflect;
use bevy_ecs::prelude::ReflectComponent;
use serde::{
    Serialize,
    Deserialize,
};
use egui::Ui;

use std::{path::PathBuf, sync::{Arc, Mutex}};

pub fn default_ui() -> Option<Arc<Mutex<Ui>>> {
    None
}

#[derive(Component, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct MainMenuComponent{
    #[serde(skip, default="default_ui")]
    #[reflect(ignore)]
    pub ui: Option<Arc<Mutex<Ui>>>
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
pub struct LeftPanelComponent{
    #[serde(skip, default="default_ui")]
    #[reflect(ignore)]
    pub ui: Option<Arc<Mutex<Ui>>>
}

impl Default for LeftPanelComponent{
    fn default() -> Self{
        LeftPanelComponent{
            ui: None,
        }
    }
}

#[derive(Component, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct RightPanelComponent{
    #[serde(skip, default="default_ui")]
    #[reflect(ignore)]
    pub ui: Option<Arc<Mutex<Ui>>>
}

impl Default for RightPanelComponent{
    fn default() -> Self{
        RightPanelComponent{
            ui: None,
        }
    }
}

#[derive(Component, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct BottomPanelComponent{
    #[serde(skip, default="default_ui")]
    #[reflect(ignore)]
    pub ui: Option<Arc<Mutex<Ui>>>
}

impl Default for BottomPanelComponent{
    fn default() -> Self{
        BottomPanelComponent{
            ui: None,
        }
    }
}

#[derive(Component, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct FileSubMenuComponent{
    #[reflect(ignore)]
    pub new_project_window: Arc<Mutex<bool>>,
    #[reflect(ignore)]
    pub open_project_window: Arc<Mutex<bool>>,
    #[reflect(ignore)]
    pub current_nav_path: PathBuf,
    #[reflect(ignore)]
    pub text_entry: String,
}

impl Default for FileSubMenuComponent {
    fn default() -> Self {
        FileSubMenuComponent{
            new_project_window: Arc::new(Mutex::new(false)),
            open_project_window: Arc::new(Mutex::new(false)),
            current_nav_path: std::env::current_dir().unwrap(),
            text_entry: std::string::String::from(""),
        }
    }
}


#[derive(Component, Reflect, Serialize, Deserialize, Default)]
#[reflect(Component)]
pub struct ComponentLibraryComponent{}

#[derive(Component, Reflect, Clone, Serialize, Deserialize)]
pub struct FileMenuSaveComponent;