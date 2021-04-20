use glium;
use glium::glutin;

// loose wrapper around the display. Saw this in a post and copied
// it but i'm honestly not sure why this is done. will mess with this later
pub struct DisplayWrapper(glium::Display);

// stores the related gluting context info
pub struct Context{
    pub event_loop: glutin::event_loop::EventLoop<()>,
    pub display: DisplayWrapper,
}

// implementation for context. mostly includes creation.
impl Context{

    // create the event loop and builders, create a display, then
    // create a context and return it with an event loop and a display
    pub fn create_new() -> Self {
        let event_loop = glutin::event_loop::EventLoop::new();
        let window_builder = glutin::window::WindowBuilder::new();
        let context_builder = glutin::ContextBuilder::new();
        let display = DisplayWrapper(
            glium::Display::new(window_builder, context_builder, &event_loop).unwrap(),
        );
        Context{
            event_loop,
            display,
        }
    }
}
