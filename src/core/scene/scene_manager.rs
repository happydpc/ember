use std::collections::HashMap;
use std::cell::{
    RefCell,
    RefMut
};
use std::sync::Mutex;
use std::path::Path;
use std::fs;
use std::convert::Infallible;

use ron;

use crate::deserialize_individually;
use crate::core::plugins::components::{
    InputComponent,
    CameraComponent,
    TransformComponent,
    TransformUiComponent,
    DebugUiComponent,
    RenderableComponent,
    DirectionalLightComponent,
    AmbientLightingComponent,
    TerrainComponent,
    TerrainUiComponent,
    SerializerFlag,
    GeometryComponent,
};
use super::{
    super::managers::manager::Manager,
    scene::{
        Scene,
        Active,
        Inactive,
        Staged,
    },
};

pub struct SceneManager{
    active_scene: Option<RefCell<Scene<Active>>>,
    active_scene_id: Option<i16>,
    staged_scene: Option<RefCell<Scene<Staged>>>,
    staged_scene_id: Option<i16>,
    scenes: Mutex<HashMap<i16, Scene<Inactive>>>, // Scene ids and scenes
    scene_counter: i16,
}

impl Manager for SceneManager{
    fn startup(&mut self){
        log::info!("Starting SceneManager...");
    }
    fn shutdown(&mut self){
        self.scenes.lock().unwrap().clear();
    }
    fn update(&mut self, _scene: &mut Scene<Active>){
    }
}

impl SceneManager{
    //
    // public API
    //

    // creates a new SceneManager
    pub fn create_new() -> Self {
        log::info!("Creating SceneManager...");
        SceneManager{
            active_scene: None,
            active_scene_id: None,
            staged_scene: None,
            staged_scene_id: None,
            scenes: Mutex::new(HashMap::new()),
            scene_counter: 0,
        }
    }

    pub fn prep_staged_scene(&mut self, _scene: &mut Scene<Staged>){
    }

    pub fn does_save_exist(&self, save_name: &'static str) -> bool {
        Path::new(save_name).exists()
    }

    // adds a scene and returns its scene id
    pub fn generate_and_register_scene(&mut self) -> i16 {
        self.scene_counter+=1;
        let key = self.scene_counter;
        self.scenes.lock().unwrap().insert(key, Scene::<Inactive>::new());
        log::info!("Registering scene {}.", key);
        key
    }

    pub fn unregister_scene(&mut self, scene_id: i16){
        self.scenes.lock().unwrap().remove(&scene_id);
        log::info!("Unregistered scene {}.", scene_id);
    }

    // gets the active scene id
    pub fn get_active_scene_id(&self) -> Option<i16> {
        self.active_scene_id.clone()
    }

    pub fn get_active_scene(&self) -> Option<RefMut<Scene<Active>>> {
        match &self.active_scene {
            Some(scene) => Some(scene.borrow_mut()),
            None => None,
        }
    }

    pub fn activate_staged_scene(&mut self){
        let staged_scene_id = self.staged_scene_id.take().expect("Staged scene id not set.");
        let staged_scene = self.staged_scene.take().expect("Staged scene not set.");
        log::info!("Activating staged scene {:?}...", staged_scene_id);
        let active_scene = Scene::<Active>::from(staged_scene.into_inner());
        self.active_scene = Some(RefCell::new(active_scene));
        self.active_scene_id = Some(staged_scene_id);
    }

    pub fn stage_scene(&mut self, id: i16){
        if self.active_scene_id.is_some() {
            log::error!("Scene {:?} does not exist to be staged.", id);
            return;
        }
        log::info!("Staging scene {:?}...", id);
        let inactive_scene = self.scenes.lock().unwrap().remove(&id).expect("Scene does not exist");
        let staged_scene = Some(RefCell::new(Scene::<Staged>::from(inactive_scene)));
        self.staged_scene = staged_scene;
        self.staged_scene_id = Some(id);
    }

    pub fn get_staged_scene(&mut self) -> Option<RefMut<Scene<Staged>>> {
        let scene = self.staged_scene.as_mut().expect("No staged scene to return.");
        Some(scene.borrow_mut())
    }

    pub fn stage_active_scene(&mut self){
        todo!();
    }

    pub fn does_scene_exist(&self, id: &i16) -> bool {
        self.scenes.lock().unwrap().contains_key(id)
    }

    pub fn load_scene_interface(&mut self, interface_path: &'static str) {
        
        let scene_id = self.generate_and_register_scene();  // create scene
        self.stage_scene(scene_id);  // stage it
    //     {
    //         let mut scene = self.get_staged_scene().expect("should probably just not wrap this?. but anyways couldn't get scene during load.");

    //         {
    //             scene.register::<TransformComponent>();
    //             scene.register::<TransformUiComponent>();
    //             scene.register::<RenderableComponent>();
    //             scene.register::<CameraComponent>();
    //             scene.register::<InputComponent>();
    //             scene.register::<DebugUiComponent>();
    //             scene.register::<DirectionalLightComponent>();
    //             scene.register::<AmbientLightingComponent>();
    //             scene.register::<TerrainComponent>();
    //             scene.register::<TerrainUiComponent>();
    //             scene.register::<GeometryComponent>();
    //             scene.register::<SimpleMarker<SerializerFlag>>();
    //             scene.insert_resource(SimpleMarkerAllocator::<SerializerFlag>::new());
    //         }

    //         let mut world = scene.get_world().expect("couldn't get world out of scene in load from ron.");


    //         {
    //             // Ensure world is empty. Can probably remove this since I literally just made it?
    //             let mut to_delete = Vec::new();
    //             for e in world.entities().join() {
    //                 to_delete.push(e);
    //             }
    //             for del in to_delete.iter() {
    //                 world.delete_entity(*del).expect("Deletion failed");
    //             }
    //         }
        
    //         let data = fs::read_to_string(interface_path).unwrap();
    //         let mut de = ron::de::Deserializer::from_str(&data).expect("Couldn't create deserializer.");
        
    //         {
    //             let mut d = (&mut world.entities(), &mut world.write_storage::<SimpleMarker<SerializerFlag>>(), &mut world.insert_resource::<SimpleMarkerAllocator<SerializerFlag>>());
        
    //             deserialize_individually!(
    //                 world, de, d,
    //                 InputComponent,
    //                 CameraComponent,
    //                 TransformComponent,
    //                 TransformUiComponent,
    //                 DebugUiComponent,
    //                 RenderableComponent,
    //                 DirectionalLightComponent,
    //                 AmbientLightingComponent,
    //                 TerrainComponent,
    //                 TerrainUiComponent,
    //                 GeometryComponent
    //             );
    //         }
        }

    //     // finally, set that scene as active
    //     // self.activate_staged_scene();
    
    //     // let mut deleteme : Option<Entity> = None;
    //     // {
    //     //     let entities = ecs.entities();
    //     //     let helper = ecs.read_storage::<SerializationHelper>();
    //     //     let player = ecs.read_storage::<Player>();
    //     //     let position = ecs.read_storage::<Position>();
    //     //     for (e,h) in (&entities, &helper).join() {
    //     //         let mut worldmap = ecs.insert_resource::<super::map::Map>();
    //     //         *worldmap = h.map.clone();
    //     //         worldmap.tile_content = vec![Vec::new(); super::map::MAPCOUNT];
    //     //         deleteme = Some(e);
    //     //     }
    //     //     for (e,_p,pos) in (&entities, &player, &position).join() {
    //     //         let mut ppos = ecs.insert_resource::<rltk::Point>();
    //     //         *ppos = rltk::Point::new(pos.x, pos.y);
    //     //         let mut player_resource = ecs.insert_resource::<Entity>();
    //     //         *player_resource = e;
    //     //     }
    //     // }
    //     // ecs.delete_entity(deleteme.unwrap()).expect("Unable to delete helper");
    // }
}
