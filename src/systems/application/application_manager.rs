use crate::systems::core::system::System;

pub struct Application{
    pub systems: Vec<Box<dyn System>>
}

impl System for Application{
    fn startup(&self){
        println!("Starting application ...")
    }
    fn shutdown(&self){
        println!("Shutting down application :");
        println!("Clearing systems... TODO : Actually clear systems lol");
    }
    fn display_system_name(&self){
        println!("application ")
    }
    fn update(&self){
        println!("Updating application ...");
        for sys in self.systems.iter(){
            sys.update();
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
    pub fn get_instance() -> Self{
        Self{
            systems: Vec::new()
        }
    }
    pub fn print_systems(&self){
        println!("application  contains: ");
        for system in self.systems.iter(){
            system.display_system_name();
        }
    }
    pub fn register_system<S: System + 'static>(&mut self, system: S) -> &mut Self{
        self.systems.push(Box::new(system));
        self
    }
    pub fn run(){
        println!("Application is running!");
    }
}
