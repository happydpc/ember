use specs::{
    World,
    RunNow,
};


pub trait SystemDispatch{
    fn run_now(&mut self, world: *mut World);
}

pub struct SingleThreadedDispatcher<'a> {
    pub systems : Vec<Box<dyn RunNow<'a>>>
}

impl<'a> SystemDispatch for SingleThreadedDispatcher<'a> {
    fn run_now(&mut self, ecs : *mut World) {
        unsafe {
            for sys in self.systems.iter_mut() {
                sys.run_now(&*ecs);
            }
        }
    }
}

pub struct MultiThreadedDispatcher {
    pub dispatcher: specs::Dispatcher<'static, 'static>
}

impl<'a> SystemDispatch for MultiThreadedDispatcher {
    fn run_now(&mut self, ecs : *mut World) {
        unsafe {
            self.dispatcher.dispatch(&mut *ecs);
            // crate::effects::run_effects_queue(&mut *ecs);
        }
    }
}

#[macro_export]
macro_rules! construct_dispatcher {
    (
        $(
            (
                $type:ident,
                $name:expr,
                $deps:expr
            )
        ),*
    ) => {
        fn new_dispatch() -> Box<dyn SystemDispatch + 'static> {
            use specs::DispatcherBuilder;

            let dispatcher = DispatcherBuilder::new()
                $(
                    .with($type{}, $name, $deps)
                )*
                .build();

            let dispatch = MultiThreadedDispatcher{
                dispatcher : dispatcher
            };

            return Box::new(dispatch);
        }
    };
}