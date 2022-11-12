use bevy::prelude::*;
use bvh_arena::{volumes::Aabb, Bvh};

type BvhResource = Bvh<Entity, Aabb<2>>;

#[derive(Debug)]
pub struct CollisionPlugin;

#[derive(Component, Debug)]
pub struct Volume {
    min: Vec2,
    max: Vec2,
}

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BvhResource>()
            .add_system_to_stage(CoreStage::PostUpdate, Self::update);
    }
}

impl CollisionPlugin {
    fn update(mut bvh: ResMut<BvhResource>, query: Query<(Entity, &Transform, &Volume)>) {
        bvh.clear();

        for (entity, &transform, volume) in &query {
            bvh.insert(entity, volume.to_aabb(transform));
        }
    }
}

impl Volume {
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    fn to_aabb(&self, transform: Transform) -> Aabb<2> {
        let pos = transform.translation.truncate();

        Aabb::from_min_max(pos + self.min, pos + self.max)
    }
}
