use std::rc::Weak;
use std::cell::RefCell;
use crate::systems::events::event::Event;

pub trait Observer{
    fn on_notify(&mut self, event: &Event);
}

pub  struct ObserverQueue{
    observers: RefCell<Vec<Box<Observer>>>,
}

impl ObserverQueue{
    fn notify(&self, event: &Event){
        let mut vector = self.observers.borrow_mut();
        for observer in vector.iter_mut() {
            observer.on_notify(event);
        }
    }
}
