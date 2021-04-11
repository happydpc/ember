// import application level structs
mod systems;

// import systems
use systems::application::application_manager::ApplicationManager;
use systems::physics::physics_system::PhysicsSystem;
use systems::rendering::render_system::RenderSystem;

// importing traits i guess
use crate::systems::core::system::System;
use glium;
use glium::Surface;

fn main() {

    // temp shit code to get a window
    use glium::glutin;
    let mut event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    event_loop.run(move |ev, _, control_flow| {

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);
        target.finish().unwrap();

        let next_frame_time = std::time::Instant::now() +
            std::time::Duration::from_nanos(16_666_667);

        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);
        match ev {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                },
                _ => return,
            },
            _ => (),
        }
    });

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
