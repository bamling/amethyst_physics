#[macro_use]
extern crate log;

pub use systems::PhysicsBundle;

use amethyst::{
    core::{math::Vector3, Float},
    ecs::{Component, DenseVecStorage, FlaggedStorage},
};
use specs_physics::bodies::Position;

mod systems;

pub struct PhysicsTransform {
    position: Vector3<Float>,
}

impl Component for PhysicsTransform {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}

impl Position<Float> for PhysicsTransform {
    fn position(&self) -> (Float, Float, Float) {
        (self.position.x, self.position.y, self.position.z)
    }

    fn set_position(&mut self, x: Float, y: Float, z: Float) {
        self.position.x = x;
        self.position.y = y;
        self.position.z = z;
    }
}

impl From<Vector3<Float>> for PhysicsTransform {
    fn from(position: Vector3<Float>) -> Self {
        Self { position }
    }
}
