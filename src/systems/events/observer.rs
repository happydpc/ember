use crate::systems::events::event::Event;

pub trait Observer<T: Event>{
    fn on_notify(&mut self, event: &T)
}
