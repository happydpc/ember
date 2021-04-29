use std::rc::Rc;
use std::cell::RefCell;
use crate::systems::core::system::System;
use crate::systems::physics::physics_system::PhysicsSystem;
use crate::systems::rendering::render_system::RenderSystem;
use crate::systems::events::event::Event;
use crate::systems::events::event_system::{Observer, Subject};

pub struct Application{
    // pub systems: Vec<Box<dyn System>>,
    state: ApplicationState,
    // own_ref: Rc<RefCell<Application>>,
}

enum ApplicationState{
    UninitializedState {},
    InitializedState {
        render_system: RefCell<RenderSystem>,
        physics_system: RefCell<PhysicsSystem>,
    },
}

impl ApplicationState{
    pub fn get_render_system(&self) -> &RefCell<RenderSystem>{
        match self{
            ApplicationState::InitializedState{render_system, ..} => render_system,
            _ => panic!("Cannot access render_system on uninitialized application."),
        }
    }
    pub fn get_physics_system(&self) -> &RefCell<PhysicsSystem>{
        match self {
            ApplicationState::InitializedState{physics_system, ..} => physics_system,
            _ => panic!("Cannot acces physics_system on uninitialized application."),
        }
    }
}

impl System for Application{
    fn startup(&mut self){
        println!("Starting application ...");
        let _state = ApplicationState::InitializedState{
            render_system: RefCell::new(RenderSystem::create_new()),
            physics_system: RefCell::new(PhysicsSystem::create_new()),
        };
        self.state = _state;
        // create sub systems
        // let mut physics_system: PhysicsSystem = PhysicsSystem::create_new();
        // let mut render_system: RenderSystem = RenderSystem::create_new();
        // startup the sub systems in order
        // TODO : consider implementing this using ECS so that systems can be quickly iterated
        // and searched
        self.state.get_physics_system().borrow_mut().startup();
        self.state.get_render_system().borrow_mut().startup();

        // register them to the application
        // self.register_system(physics_system);
        // self.register_system(render_system);

        // register self as observer to relevant systems
        self.register_with_subjects();

    }
    fn shutdown(&mut self){
        println!("Shutting down application...");
        // TODO : Definitely find a better way to access the systems
        self.state.get_physics_system().borrow_mut().shutdown();
        self.state.get_render_system().borrow_mut().shutdown();
    }
    fn update(&self){
        // TODO : Will the core app update do anything? should run just call update on loop
        // and then have this iterate over the systems and update? seems like an unecessary
        // layer to have the run function just be a thin wrapper around this.
        println!("Updating application ...");
        self.state.get_physics_system().borrow_mut().update();
        self.state.get_render_system().borrow_mut().update();
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
        let _state = ApplicationState::UninitializedState{};
        Self{
            state: _state
        }
    }
    // pub fn register_system<S: System + 'static>(&mut self, system: S) -> &mut Self{
    //     self.systems.push(Box::new(system));
    //     self
    // }
    pub fn run(&self){
        println!("Running the application...");
        self.state.get_render_system().borrow_mut().run();
    }
    fn register_with_subjects(&self){
        // self.state.get_render_system().window.context.register(Rc::new(RefCell::(self)));
    }
}
