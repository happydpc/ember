pub trait System{
    fn startup(&mut self);
    fn shutdown(&mut self);
    fn update(&mut self);
}
