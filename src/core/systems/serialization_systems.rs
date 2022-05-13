use crate::core::plugins::components::{
    DebugUiComponent,
};
use ron::ser::{to_string_pretty, PrettyConfig};
use bevy_ecs::prelude::{
    ResMut,
};
use bevy_ecs::event::{
    // Reader,
    Events,
};
use crate::core::events::serialization_events::SaveEvent;

// types
pub struct SerializerData(Vec<String>);

pub fn SceneSerializationSystem(
    mut save_events: ResMut<Events<SaveEvent>>,
){
    let mut reader = save_events.get_reader();
    for event in reader.iter(&save_events){
        log::info!("Save detected!");
    }
    save_events.clear();
}


// systems
// pub struct MenuSerializerSystem;
// impl<'a> System<'a> for MenuSerializerSystem{
//     type SystemData = (
//         Read<'a, DebugUiComponent>,
//         WriteExpect<'a, SerializerData>,
//     );

//     pub fn run(&mut self, data: Self::SystemData){
//         (let comps, mut serializer_vec) = data;
//         let pretty = PrettyConfig::new()
//             .depth_limit(2)
//             .separate_tuple_members(true)
//             .enumerate_arrays(true);
//         for comp in comps.join() {
//             serializer_vec.push(to_string_pretty(&comp, pretty.clone).expect("Couldn't serialize menu."));
//         }
//     }
// }

// // systems
// pub struct SerializerSystem;
// impl<'a> System<'a> for SerializerSystem{
//     type SystemData = (
//         Entities<'a>,
//         Read<'a, LazyUpdate>,
//     );

//     pub fn run(&mut self, data: Self::SystemData){
//         // (let entities, mut serializer_vec) = data;
//         // let pretty = PrettyConfig::new()
//             // .depth_limit(2)
//             // .separate_tuple_members(true)
//             // .enumerate_arrays(true);
//         // for comp in comps.join() {
//             // serializer_vec.push(to_string_pretty(&comp, pretty.clone).expect("Couldn't serialize menu."));
//         // }

//     }
// }