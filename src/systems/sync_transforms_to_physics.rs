use amethyst::{
    core::{transform::Transform, Float},
    ecs::{
        storage::ComponentEvent,
        Entities,
        Join,
        ReadStorage,
        ReaderId,
        Resources,
        System,
        SystemData,
        WriteStorage,
    },
};
use specs_physics::{bodies::Position, Physics};

use crate::PhysicsTransform;

use super::iterate_component_events;

#[derive(Default)]
pub struct SyncTransformsToPhysicsSystem {
    transforms_reader_id: Option<ReaderId<ComponentEvent>>,
    // TODO: inserted PhysicsBody/PhysicsCollider
}

impl<'s> System<'s> for SyncTransformsToPhysicsSystem {
    type SystemData = (
        Entities<'s>,
        ReadStorage<'s, Transform>,
        WriteStorage<'s, PhysicsTransform>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, transforms, mut physics_transforms) = data;

        let (inserted_transforms, modified_transforms, removed_transforms) =
            iterate_component_events(&transforms, self.transforms_reader_id.as_mut().unwrap());

        for (entity, transform, id) in (
            &entities,
            &transforms,
            &inserted_transforms | &modified_transforms | &removed_transforms,
        )
            .join()
        {
            // handle inserted events
            if inserted_transforms.contains(id) {
                debug!("Inserted Transform with id: {}", id);
                if let Err(err) = physics_transforms
                    .insert(entity, PhysicsTransform::from(*transform.translation()))
                {
                    warn!("Failed to insert PhysicsTransform: {}", err);
                }
            }

            //// handle modified events
            //if modified_transforms.contains(id) {
            //    debug!("Modified Transform with id: {}", id);
            //    if let Some(physics_transform) = physics_transforms.get_mut(entity) {
            //        physics_transform.set_position(
            //            transform.translation().x,
            //            transform.translation().y,
            //            transform.translation().z,
            //        );
            //    }
            //}

            // handle removed events
            if removed_transforms.contains(id) {
                debug!("Removed Transform with id: {}", id);
                if let Some(_) = physics_transforms.remove(entity) {
                    info!("Removed PhysicsTransform with id: {}", id);
                }
            }
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        info!("SyncTransformsToPhysicsSystem.setup");
        Self::SystemData::setup(res);

        // initialise required resources
        res.entry::<Physics<Float>>()
            .or_insert_with(Physics::default);

        // register reader id for the Transform storage
        let mut transform_storage: WriteStorage<Transform> = SystemData::fetch(&res);
        self.transforms_reader_id = Some(transform_storage.register_reader());
    }
}
