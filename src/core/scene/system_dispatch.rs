// use bevy_ecs::world::World;
// use bevy_ecs::schedule::Stage;


// pub trait Systemschedule{
//     fn run_now(&mut self, world: *mut World);
// }

// pub struct MultiThreadedscheduleer {
//     pub scheduleer: specs::scheduleer<'static, 'static>
// }

// impl<'a> Systemschedule for MultiThreadedscheduleer {
//     fn run_now(&mut self, ecs : *mut World) {
//         unsafe {
//             self.scheduleer.schedule(&mut *ecs);
//             // crate::effects::run_effects_queue(&mut *ecs);
//         }
//     }
// }

// #[macro_export]
// macro_rules! construct_single_stage_schedule {
//     (
//         $(
//             (
//                 $type:ident,
//                 $name:expr,
//                 $deps:expr
//             )
//         ),*
//     ) => {
//         fn new_single_stage_schedule() -> Box<dyn Stage>{// + 'static> {

//             let mut schedule = Schedule::default()
//                 $(
//                     .add_stage("main", $type{}, $name, $deps)
//                 )*
//                 .build();

//             let schedule = MultiThreadedscheduleer{
//                 scheduleer : scheduleer
//             };

//             return Box::new(schedule);
//         }
//     };
// }