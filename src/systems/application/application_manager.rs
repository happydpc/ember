use crate::systems::core::system::System;
use crate::systems::physics::physics_system::PhysicsSystem;
use crate::systems::rendering::render_system::RenderSystem;
use crate::systems::events::event::Event;
use crate::systems::events::event_system::Observer;

pub struct Application{
    pub systems: Vec<Box<dyn System>>,
}

impl System for Application{
    fn startup(&mut self){
        println!("Starting application ...");
        // create sub systems
        let mut physics_system: PhysicsSystem = PhysicsSystem::create_new();
        let mut render_system: RenderSystem = RenderSystem::create_new();
        // startup the sub systems in order
        // TODO : consider implementing this using ECS so that systems can be quickly iterated
        // and searched
        physics_system.startup();
        render_system.startup();
        // register them to the application
        self.register_system(physics_system);
        self.register_system(render_system);
    }
    fn shutdown(&mut self){
        println!("Shutting down application...");
        // TODO : Definitely find a better way to access the systems
        self.systems[0].shutdown();
        self.systems[1].shutdown();
        self.systems.clear();
    }
    fn update(&self){
        // TODO : Will the core app update do anything? should run just call update on loop
        // and then have this iterate over the systems and update? seems like an unecessary
        // layer to have the run function just be a thin wrapper around this.
        println!("Updating application ...");
        for sys in self.systems.iter(){
            sys.update();
        }
    }
}

impl Observer for Application{
    fn on_notify(&mut self, event: &Event){
        match event{
            Event::ContextUpdate => {
                println!("Received a context update!");
            }
        }
    }
}

impl Application{
    // called by the client when they want to create an application
    pub fn create_application() -> Self{
        Self{
            systems: Vec::new()
        }
    }
    pub fn register_system<S: System + 'static>(&mut self, system: S) -> &mut Self{
        self.systems.push(Box::new(system));
        self
    }
    pub fn run(&self){
        println!("Running the application...");
        loop{
            for sys in self.systems.iter(){
                sys.update();
            }
        };
    }
}
