use amethyst::{
    core::{bundle::SystemBundle, Float},
    ecs::{
        storage::{ComponentEvent, MaskedStorage},
        BitSet,
        Component,
        DispatcherBuilder,
        ReaderId,
        Storage,
        Tracked,
    },
    error::Error,
};
use std::ops::Deref;

use self::{
    debug::DebugSystem,
    sync_transforms_from_physics::SyncTransformsFromPhysicsSystem,
    sync_transforms_to_physics::SyncTransformsToPhysicsSystem,
};

use specs_physics::register_physics_systems;

use crate::PhysicsTransform;

mod debug;
mod sync_transforms_from_physics;
mod sync_transforms_to_physics;

#[derive(Default)]
pub struct PhysicsBundle {
    debug_lines: bool,
}

impl<'a, 'b> SystemBundle<'a, 'b> for PhysicsBundle {
    fn build(self, dispatcher: &mut DispatcherBuilder) -> Result<(), Error> {
        dispatcher.add(
            SyncTransformsToPhysicsSystem::default(),
            "sync_transforms_to_physics_system",
            &[],
        );

        register_physics_systems::<Float, PhysicsTransform>(dispatcher);

        dispatcher.add(
            SyncTransformsFromPhysicsSystem::default(),
            "sync_transforms_from_physics_system",
            &["sync_positions_from_physics_system"],
        );

        if self.debug_lines {
            dispatcher.add(
                DebugSystem::default(),
                "debug_system",
                &["sync_transforms_from_physics_system"],
            );
        }

        Ok(())
    }
}

impl PhysicsBundle {
    /// Enables the `DebugSystem` which draws `DebugLines` around
    /// `PhysicsCollider` shapes.
    pub fn with_debug_lines(mut self) -> Self {
        self.debug_lines = true;
        self
    }
}

pub(crate) fn iterate_component_events<T, D>(
    tracked_storage: &Storage<T, D>,
    reader_id: &mut ReaderId<ComponentEvent>,
) -> (BitSet, BitSet, BitSet)
where
    T: Component,
    T::Storage: Tracked,
    D: Deref<Target = MaskedStorage<T>>,
{
    let (mut inserted, mut modified, mut removed) = (BitSet::new(), BitSet::new(), BitSet::new());
    for component_event in tracked_storage.channel().read(reader_id) {
        match component_event {
            ComponentEvent::Inserted(id) => {
                debug!("Got Inserted event with id: {}", id);
                inserted.add(*id);
            }
            ComponentEvent::Modified(id) => {
                debug!("Got Modified event with id: {}", id);
                modified.add(*id);
            }
            ComponentEvent::Removed(id) => {
                debug!("Got Removed event with id: {}", id);
                removed.add(*id);
            }
        }
    }

    (inserted, modified, removed)
}
