
use crate::core::scene::DynamicScene;
use crate::core::managers::SceneManagerMessagePump;
use crate::core::events::scene_manager_messages::SceneManagerMessage;

use std::fs::File;
use bevy_ecs::prelude::{
    Query, 
    World,
    Res,
    ResMut,
};
use bevy_ecs::entity::{
    Entity,
};

use bevy_ecs::prelude::EventReader;

use crate::core::scene::TypeRegistryResource;

use crate::core::events::project_events::{SaveEvent, CreateProjectEvent, OpenProjectEvent};

// types
pub struct SerializerData(Vec<String>);

pub fn SceneSerializationSystem(
    world: &World,
    _query: Query<Entity>,
    mut save_events: EventReader<SaveEvent>,
    type_registry: Res<TypeRegistryResource>,
){
    for _event in save_events.iter(){
        let scene = DynamicScene::from_world(&world, &type_registry.0);
        scene.write_to_file("./new_save.ron", &type_registry.0);
    }
    save_events.clear();
}

pub fn ProjectCreationSystem(
    mut new_project_events: EventReader<CreateProjectEvent>,
    mut scene_manager_messages: ResMut<SceneManagerMessagePump>
){
    for event in new_project_events.iter() {
        log::info!("Creating a project");

        std::fs::create_dir(event.project_path.clone()).expect("Couldn't create project");
        let mut scenes_folder = format!("{}/scenes", event.project_path.clone());
        std::fs::create_dir(scenes_folder.clone());

        // basically run cargo init here

        scenes_folder.push_str("/default.ron");
        match File::create(&scenes_folder) {
            Err(why) => panic!("couldn't create default ron scene: {}", why),
            Ok(_file) => (),
        };
        let mut project_file = event.project_path.clone();
        project_file.push_str("/ember.project");
        match File::create(&project_file){
            Err(why) => panic!("couldn't create project file {}", why),
            Ok(_file) => (),
        };
        
        let m = SceneManagerMessage::OpenProject {
            path: event.project_path.clone(),
            scene_name: String::from("default.ron")
        };
        scene_manager_messages.send(m);
    }
    new_project_events.clear();
}

pub fn OpenProjectSystem(
    mut open_project_events: EventReader<OpenProjectEvent>,
    mut scene_manager_messages: ResMut<SceneManagerMessagePump>
){
    for event in open_project_events.iter() {
        log::info!("Opening a project");
        let m = SceneManagerMessage::OpenProject {
            path: event.project_path.clone(),
            scene_name: String::from("default.ron")
        };
        scene_manager_messages.send(m);
    }
    open_project_events.clear();
}