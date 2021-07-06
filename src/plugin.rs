use amethyst_core::ecs::{
    Resources,
    World,
};
use amethyst_error::Error;
use amethyst_rendy::{
    bundle::{
        RenderOrder,
        RenderPlan,
        Target,
    },
    Backend,
    Factory,
    RenderGroupDesc,
    RenderPlugin,
};

use crate::pass::DrawEguiDesc;

#[derive(Default, Debug)]
pub struct RenderEgui {
    target: Target,
}

impl RenderEgui {
    /// Select render target on which UI should be rendered.
    pub fn with_target(mut self, target: Target) -> Self {
        self.target = target;
        self
    }
}

impl<B: Backend> RenderPlugin<B> for RenderEgui {
    fn on_plan(
        &mut self,
        plan: &mut RenderPlan<B>,
        _factory: &mut Factory<B>,
        _world: &World,
        _resources: &Resources,
    ) -> Result<(), Error> {
        plan.extend_target(self.target, |ctx| {
            ctx.add(RenderOrder::Overlay, DrawEguiDesc::default().builder())?;
            Ok(())
        });
        Ok(())
    }
}
