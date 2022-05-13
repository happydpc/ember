use bevy_ecs::component::Component;
use serde::{
    Serialize,
    Deserialize,
};
use egui::Ui;
use bevy_ecs::entity::Entity;

#[derive(Component, Serialize, Deserialize)]
pub struct MainMenuComponent{
    #[serde(skip, default="MainMenuComponent::default_ui")]
    pub ui: Option<Ui>
}

impl MainMenuComponent{
    pub fn default_ui() -> Option<Ui> {
        None
    }
}

#[derive(Component, Clone, Serialize, Deserialize)]
pub struct FileSubMenuComponent{
    // #[serde(skip, default="MainMenuComponent::default_ui")]
    pub parent: Option<Entity>
}

#[derive(Component, Clone, Serialize, Deserialize)]
pub struct FileMenuSaveComponent;