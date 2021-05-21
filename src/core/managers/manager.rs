pub trait Manager{
    fn startup(&mut self);
    fn shutdown(&mut self);
    fn update(&mut self);
}
