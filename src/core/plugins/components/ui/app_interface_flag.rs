use specs::{Component, HashMapStorage};
use serde::{
    Serialize,
    Deserialize,
};

#[derive(Component, Clone, Serialize, Deserialize)]
#[storage(HashMapStorage)]
pub struct AppInterfaceFlag;
