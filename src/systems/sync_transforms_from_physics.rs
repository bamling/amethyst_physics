use amethyst::{
    core::{transform::Transform, Float},
    ecs::{
        storage::ComponentEvent,
        Join,
        ReadStorage,
        ReaderId,
        Resources,
        System,
        SystemData,
        WriteStorage,
    },
};

use super::iterate_component_events;
use crate::PhysicsTransform;

use specs_physics::{bodies::Position, Physics};

#[derive(Default)]
pub struct SyncTransformsFromPhysicsSystem {
    physics_transforms_reader_id: Option<ReaderId<ComponentEvent>>,
}

impl<'s> System<'s> for SyncTransformsFromPhysicsSystem {
    type SystemData = (
        ReadStorage<'s, PhysicsTransform>,
        WriteStorage<'s, Transform>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (physics_transforms, mut transforms) = data;

        let (_, modified_physics_transforms, _) = iterate_component_events(
            &physics_transforms,
            self.physics_transforms_reader_id.as_mut().unwrap(),
        );

        for (physics_transforms, transform, _) in (
            &physics_transforms,
            &mut transforms,
            &modified_physics_transforms,
        )
            .join()
        {
            let (x, y, z) = physics_transforms.position();
            transform.set_translation_xyz(x, y, z);
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        info!("SyncTransformsFromPhysicsSystem.setup");
        Self::SystemData::setup(res);

        // initialise required resources
        res.entry::<Physics<Float>>()
            .or_insert_with(Physics::default);

        // register reader id for the PhysicsTransform storage
        let mut physics_transform_storage: WriteStorage<PhysicsTransform> = SystemData::fetch(&res);
        self.physics_transforms_reader_id = Some(physics_transform_storage.register_reader());
    }
}
