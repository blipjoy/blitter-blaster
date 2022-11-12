use crate::engine::bitmap::Bitmap;
use bevy::prelude::*;
use bvh_arena::{volumes::Aabb, Bvh};

pub type BvhResource = Bvh<Entity, Aabb<2>>;

#[derive(Debug)]
pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BvhResource>()
            .add_system_to_stage(CoreStage::PostUpdate, Self::update);
    }
}

impl CollisionPlugin {
    fn update(mut bvh: ResMut<BvhResource>, query: Query<(Entity, &Bitmap, &Transform)>) {
        bvh.clear();

        for (entity, bitmap, &transform) in &query {
            bvh.insert(entity, bitmap.to_aabb(transform));
        }
    }
}

impl Bitmap {
    fn to_aabb(&self, transform: Transform) -> Aabb<2> {
        let pos = transform.translation.truncate();
        let size = Vec2::new(self.width() as f32, self.height() as f32);

        Aabb::from_min_max(pos, pos + size)
    }
}
