use bevy_ecs::{component::Component, prelude::Entity, system::Resource};


use bevy_reflect::{Reflect, FromReflect, ReflectSerialize, ReflectDeserialize};
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

#[derive(Reflect, FromReflect, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[reflect_value(PartialEq, Serialize, Deserialize)]
pub enum PanelType{
    left,
    right,
    top,
    bottom,
}

#[derive(Resource)]
pub struct EditorUiState{
    pub selected_entity: Option<Entity>,
    pub new_project_window_open: bool,
    pub open_project_window_open: bool,
}
impl Default for EditorUiState{
    fn default() -> Self {
        EditorUiState {
            selected_entity: None,
            new_project_window_open: false,
            open_project_window_open: false
        }
    }
}

#[derive(Component, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct UiPanelComponent{
    pub panel_type: PanelType,
    #[serde(skip, default="default_ui")]
    #[reflect(ignore)]
    pub ui: Option<Arc<Mutex<Ui>>>
}

impl UiPanelComponent{
    #[inline]
    pub fn left() -> Self {
        UiPanelComponent { panel_type: PanelType::left, ui: None }
    }

    #[inline]
    pub fn right() -> Self {
        UiPanelComponent { panel_type: PanelType::right, ui: None }
    }

    #[inline]
    pub fn top() -> Self {
        UiPanelComponent { panel_type: PanelType::top, ui: None }
    }

    #[inline]
    pub fn bottom() -> Self {
        UiPanelComponent { panel_type: PanelType::bottom, ui: None }
    }
}


impl Default for UiPanelComponent{
    fn default() -> Self{
        UiPanelComponent{
            panel_type: PanelType::right,
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

#[derive(Component, Reflect, Serialize, Deserialize, Default)]
#[reflect(Component)]
pub struct EntityInspectorComponent{}


#[derive(Component, Reflect, Clone, Serialize, Deserialize)]
pub struct FileMenuSaveComponent;