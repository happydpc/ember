use crate::systems::core::system::System;

pub struct ApplicationManager{
    pub systems: Vec<Box<dyn System>>
}

impl System for ApplicationManager{
    fn startup(&self){
        println!("Starting application manager...")
    }
    fn shutdown(&self){
        println!("Shutting down application manager:");
        println!("Clearing systems... TODO : Actually clear systems lol");
    }
    fn display_system_name(&self){
        println!("Application Manager")
    }
    fn update(&self){
        println!("Updating application manager...");
        for sys in self.systems.iter(){
            sys.update();
        }
    }
}

impl ApplicationManager{
    pub fn get_instance() -> Self{
        Self{
            systems: Vec::new()
        }
    }
    pub fn print_systems(&self){
        println!("Application Manager contains: ");
        for system in self.systems.iter(){
            system.display_system_name();
        }
    }
    pub fn register_system<S: System + 'static>(&mut self, system: S) -> &mut Self{
        self.systems.push(Box::new(system));
        self
    }
}
