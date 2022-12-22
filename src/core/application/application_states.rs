use crate::core::scene::{
    Scene,
    Active,
    Staged
};

use bevy_ecs::{
    schedule::Stage,
};
use bevy_ecs::prelude::Schedule;




use crate::core::plugins::components::{
    DebugUiComponent,
    CameraComponent,
    InputComponent,
    MainMenuComponent,
    FileSubMenuComponent,
    TransformComponent,
};
use crate::core::events::project_events::SaveEvent;
use crate::core::events::project_events::CreateProjectEvent;
use crate::core::events::project_events::CloseProjectEvent;
use crate::core::events::project_events::OpenProjectEvent;
use crate::core::events::menu_messages::MenuMessage;

use crate::core::managers::SceneManagerMessagePump;
use crate::core::plugins::components::AppInterfaceFlag;

use bevy_ecs::event::Events;
use bevy_reflect::TypeRegistryArc;



pub trait ApplicationState{
    fn run_schedule(&mut self, scene: &mut Scene<Active>);
    fn init_schedule(&mut self);
    fn scene_interface_path(&self) -> &'static str;
    fn overlay_interface_on_staged_scene(&self, scene: &mut Scene<Staged>);
}

pub struct ApplicationIdleState{
    pub schedule: Option<Box<dyn Stage>>,
    pub scene_interface_path: &'static str,
}

impl ApplicationIdleState{
    pub fn create() -> Self{
        ApplicationIdleState{
            schedule: None,
            scene_interface_path: "./idle_state.ron",
        }
    }
}

impl ApplicationState for ApplicationIdleState {
    fn run_schedule(&mut self, scene: &mut Scene<Active>){
        log::info!("Running scene schedule...");
        let mut schedule = self.schedule.take().expect("No setup schedule");
        schedule.run(&mut *scene.get_world().unwrap());
        self.schedule = Some(schedule);
    }

    fn init_schedule(&mut self){
        let schedule = Schedule::default();
        self.schedule = Some(Box::new(schedule));
    }
    
    fn scene_interface_path(&self) -> &'static str{
        self.scene_interface_path
    }

    fn overlay_interface_on_staged_scene(&self, scene: &mut Scene<Staged>){
        scene.get_world()
            .unwrap()
            .init_resource::<Events<SaveEvent>>();

        scene.get_world()
            .unwrap()
            .init_resource::<TypeRegistryArc>();

        scene.get_world()
            .unwrap()
            .init_resource::<SceneManagerMessagePump>();

        scene.get_world()
            .unwrap()
            .init_resource::<Events<CreateProjectEvent>>();

        scene.get_world()
            .unwrap()
            .init_resource::<Events<CloseProjectEvent>>();

        scene.get_world()
            .unwrap()
            .init_resource::<Events<OpenProjectEvent>>();

        scene.get_world()
            .unwrap()
            .init_resource::<Events<MenuMessage<MainMenuComponent>>>();
        scene.get_world()
            .unwrap()
            .init_resource::<Events<MenuMessage<FileSubMenuComponent>>>();

        {
            let mut world = scene.get_world().unwrap();
            let registry_arc = world.get_resource_mut::<TypeRegistryArc>().unwrap();
            let mut registry = registry_arc.write();
            registry.register::<AppInterfaceFlag>();
            registry.register::<MainMenuComponent>();
            registry.register::<DebugUiComponent>();
            registry.register::<FileSubMenuComponent>();
            registry.register::<CameraComponent>();
            registry.register::<InputComponent>();
        }

        let _MainMenuEntity = scene.get_world()
            .unwrap()
            .spawn()
            .insert(AppInterfaceFlag{})
            .insert(MainMenuComponent{ui: None})
            .insert(DebugUiComponent::create())
            .insert(FileSubMenuComponent::new())    
            .id();

        scene.get_world()
            .unwrap()
            .spawn()
            .insert(CameraComponent::create_default())
            .insert(TransformComponent::create_empty())
            .insert(InputComponent::create())
            .id();
    }
    
}

pub struct ApplicationEditorState{
    pub schedule: Option<Box<dyn Stage>>,
    pub scene_interface_path: &'static str,
}

