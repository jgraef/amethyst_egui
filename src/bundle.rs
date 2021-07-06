use amethyst_core::{
    dispatcher::DispatcherBuilder,
    ecs::{
        Resources,
        SystemBundle,
        World,
    },
    EventChannel,
};
use amethyst_error::Error;
use winit::event::Event;

use crate::system::{
    EguiConfig,
    EguiContext,
    EguiInputGrab,
    EguiSystem,
};

#[derive(Debug, Default)]
pub struct EguiBundle;

impl SystemBundle for EguiBundle {
    fn load(
        &mut self,
        _world: &mut World,
        resources: &mut Resources,
        builder: &mut DispatcherBuilder,
    ) -> Result<(), Error> {
        resources.insert(EguiInputGrab::default());
        resources.insert(EguiConfig::default());
        resources.insert(EguiContext::default());

        /*let mut window_events = resources
            .get_mut::<EventChannel<WindowEvent<'static>>>()
            .expect("EventChannel<WindowEvent>> missing");
        let window_event_reader = window_events.register_reader();*/

        let winit_event_reader = resources
            .get_mut::<EventChannel<Event<'_, ()>>>()
            .expect("Window event channel not found in resources")
            .register_reader();

        builder.add_system(EguiSystem::new(winit_event_reader));

        Ok(())
    }
}
