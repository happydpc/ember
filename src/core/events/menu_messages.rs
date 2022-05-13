use egui::Ui;
use std::ops::FnOnce;
use std::sync::{Arc, Mutex};

pub struct MenuMessage<T>{
    target: Option<T>,
    // pub ui: fn(&mut Ui),
    pub ui: Box<dyn FnOnce(&mut Ui) + Send + Sync + 'static>,
}

impl<T> MenuMessage<T>{
    pub fn new<F: FnOnce(&mut Ui) + Send + Sync + 'static>(ui: F) -> Self {
        Self {
            target: None,
            ui: Box::new(ui),
        }
    }
}