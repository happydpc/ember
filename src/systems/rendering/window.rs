pub trait Window{
    fn get_width() -> i16;
    fn get_height() -> i16;
    fn on_update();
    fn set_event_callback();
    fn create_new() -> Self;
}
