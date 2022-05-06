use specs::{System, Read};
use crate::core::plugins::components::{
    DebugUiComponent,
};
use ron::ser::{to_string_pretty, PrettyConfig};


// types
pub struct SerializerData(Vec<String>);


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