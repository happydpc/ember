pub trait System{
    fn startup(&mut self);
    fn shutdown(&mut self);
    fn display_system_name(&self);
    fn update(&self);
}
