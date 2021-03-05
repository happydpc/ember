// import application level structs
mod systems;

// import systems
use systems::application::application_manager::ApplicationManager;
use systems::physics::physics_system::PhysicsSystem;
use systems::rendering::render_system::RenderSystem;

// importing traits i guess
use crate::systems::core::system::System;


fn main() {
    // create everything
    let mut app: ApplicationManager = ApplicationManager::get_instance();
    let physics_system: PhysicsSystem = PhysicsSystem{};
    let render_system: RenderSystem = RenderSystem{};

    // startup the application manager
    app.startup();

    // registering systems with the application manager
    app.register_system(physics_system);
    app.register_system(render_system);

    // run startup process
    for sys in app.systems.iter(){
        sys.startup();
    }

    // main application loop
    let mut count: i32 = 0;
    loop{
        app.update();

        if count == 10{
            break;
        }
        count +=1;
    }

    // run shutdown process
    for sys in app.systems.iter(){
        sys.shutdown();
    }

    // finally shut down the application manager
    app.shutdown();

}
