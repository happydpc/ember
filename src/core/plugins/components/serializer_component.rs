use specs::{Component, VecStorage};
use serde::{
    Serialize,
    Deserialize,
};

#[derive(Component, Clone, Serialize, Deserialize)]
#[storage(VecStorage)]
pub struct SerializerFlag;