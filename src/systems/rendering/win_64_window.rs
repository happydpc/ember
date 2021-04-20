use crate::systems::rendering::window::Window;
use crate::systems::rendering::context::Context;
use glium;

pub struct Win64Window{
    pub context: Context,
}

impl Window for Win64Window {

    fn get_width() -> i16{
        32
    }
    fn get_height() -> i16{
        32
    }
    fn on_update() {
        // i think here I want the context to run and send a message on each frame update to any
        // listeners (in this case the main application) and that then calls the update there.

    }
    fn set_event_callback() {

    }
    fn create_window() -> Win64Window {
        let mut win: Win64Window = Win64Window{
            context: Context::create_new(),
        };

        win.init();
        win
    }
}

impl Win64Window {
    fn init(&mut self){
    }
}
