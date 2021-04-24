use std::cell::RefCell;
use std::rc::Rc;
use crate::systems::events::event::Event;

// Observers register with subjects and wait for events
pub trait Observer{
    fn on_notify(&mut self, event: &Event);
}

// Subject stores observers and sends them events
// had to use a reference counter and store the actual observer in a RefCell
// so that we can call methods on the observer without the subject owning it.
pub trait Subject{
    fn register(&mut self, observer: Rc<RefCell<dyn Observer>>);
    fn notify(&mut self, event: &Event);
}
