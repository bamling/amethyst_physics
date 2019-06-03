use amethyst::{
    core::{transform::Transform, Float},
    ecs::{Join, ReadStorage, Resources, System, SystemData, Write},
    renderer::{
        debug_drawing::{DebugLines, DebugLinesParams},
        palette::Srgba,
    },
};

use specs_physics::colliders::{PhysicsCollider, Shape};

/// The `DebugSystem`s handles the drawing of `DebugLines` elements for
/// `PhysicsCollider`s. This visualises the `PhysicsCollider` and enables easier
/// debugging of collisions.
#[derive(Default)]
pub struct DebugSystem;

impl<'s> System<'s> for DebugSystem {
    type SystemData = (
        ReadStorage<'s, Transform>,
        ReadStorage<'s, PhysicsCollider<Float>>,
        Write<'s, DebugLines>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (transforms, physics_colliders, mut debug_lines) = data;

        // iterate over PhysicsColliders and their Transforms and draw lines accordingly
        for (transform, physics_collider) in (&transforms, &physics_colliders).join() {
            // fetch the parent for its position
            let (x, y, z) = (
                transform.translation().x.as_f32(),
                transform.translation().y.as_f32(),
                transform.translation().z.as_f32(),
            );

            // color based on type
            let color = if physics_collider.sensor {
                Srgba::new(0.13, 0.65, 0.94, 1.0) // 1 or 1/255?!
            } else {
                Srgba::new(0.81, 0.0, 0.5, 1.0) // 1 or 1/255?!
            };

            // depending on the Shape we draw the DebugLines differently; right now we only
            // support Shape::Rectangle
            match physics_collider.shape {
                Shape::Rectangle(width, height, _) => {
                    let width = width.as_f32();
                    let height = height.as_f32();

                    // draw top line
                    debug_lines.draw_line(
                        [x - width / 2.0, y + height / 2.0, z].into(),
                        [x + width / 2.0, y + height / 2.0, z].into(),
                        color,
                    );

                    // draw right line
                    debug_lines.draw_line(
                        [x + width / 2.0, y + height / 2.0, z].into(),
                        [x + width / 2.0, y - height / 2.0, z].into(),
                        color,
                    );

                    // draw bottom line
                    debug_lines.draw_line(
                        [x + width / 2.0, y - height / 2.0, z].into(),
                        [x - width / 2.0, y - height / 2.0, z].into(),
                        color,
                    );

                    // draw bottom line
                    debug_lines.draw_line(
                        [x - width / 2.0, y - height / 2.0, z].into(),
                        [x - width / 2.0, y + height / 2.0, z].into(),
                        color,
                    );
                }
                _ => {}
            }
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        info!("DebugSystem.setup");
        Self::SystemData::setup(res);

        // initialise required resources
        res.entry::<DebugLines>().or_insert(DebugLines::new());
        res.entry::<DebugLinesParams>()
            .or_insert(DebugLinesParams { line_width: 1.0 });
    }
}
