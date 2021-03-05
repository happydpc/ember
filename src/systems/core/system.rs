pub trait System{
    fn startup(&self);
    fn shutdown(&self);
    fn display_system_name(&self);
    fn update(&self);
}
